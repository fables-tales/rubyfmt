use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus};

type Output = Result<(), Box<dyn Error>>;

struct RubyConfig {
    libz: bool,
    libcrypt: bool,
}

fn main() -> Output {
    #[cfg(target_os = "linux")]
    let libname = "ruby-static";
    #[cfg(target_os = "macos")]
    let libname = "ruby.3.2-static";
    #[cfg(all(target_arch = "x86_64", windows))]
    let libname = "x64-vcruntime140-ruby320-static";
    #[cfg(all(target_arch = "x86", windows))]
    let libname = "vcruntime140-ruby320-static";
    #[cfg(all(target_env = "gnu", windows))]
    compile_error!("rubyfmt on Windows is currently only supported with msvc");

    #[cfg(unix)]
    let ripper = "ext/ripper/ripper.o";
    #[cfg(windows)]
    let ripper = "ext/ripper/ripper.obj";

    let path = env::current_dir()?;
    let ruby_checkout_path = path.join("ruby_checkout");

    let old_checkout_sha = if ruby_checkout_path.join(ripper).exists() {
        Some(get_ruby_checkout_sha())
    } else {
        None
    };

    let _ = Command::new("git")
        .args(&["submodule", "update", "--init"])
        .status();

    let new_checkout_sha = get_ruby_checkout_sha();

    // Only rerun this build if the ruby_checkout has changed
    match old_checkout_sha {
        Some(old_sha) if old_sha == new_checkout_sha => {}
        _ => {
            make_configure(&ruby_checkout_path)?;
            run_configure(&ruby_checkout_path)?;
            build_ruby(&ruby_checkout_path)?;
        }
    }

    let arch = extract_ruby_arch(&ruby_checkout_path);

    cc::Build::new()
        .file("src/rubyfmt.c")
        .object(ruby_checkout_path.join(&ripper))
        .include(ruby_checkout_path.join("include"))
        .include(ruby_checkout_path.join(format!(".ext/include/{}", arch)))
        .warnings(false)
        .compile("rubyfmt_c");

    println!(
        "cargo:rustc-link-search=native={}",
        ruby_checkout_path.display()
    );
    println!("cargo:rustc-link-lib=static={}", libname);

    let config = extract_ruby_library_config(&ruby_checkout_path, &arch);
    if config.libz {
        println!("cargo:rustc-link-lib=dylib=z");
    }
    if config.libcrypt {
        println!("cargo:rustc-link-lib=dylib=crypt");
    }

    Ok(())
}

fn extract_ruby_arch(ruby_checkout_path: &Path) -> String {
    let rbconfig_rb = ruby_checkout_path.join("rbconfig.rb");
    let f = File::open(rbconfig_rb).expect("cannot find rbconfig.rb");
    let f = BufReader::new(f);
    // Naturally, rbconfig.rb permits all manner of Ruby syntax to be used
    // in the values for CONFIG.  Matching arbitrary Ruby inside the value
    // string via [^"]+ could be a recipe for very confusing error
    // messages later.  So we deliberately limit the charcters in the
    // value string here.
    let arch_regex = Regex::new("  CONFIG\\[\"arch\"\\] = \"(?P<arch>[-a-z0-9_]+)\"")
        .expect("incorrect regex syntax");
    for line in f.lines() {
        let line = line.expect("could not read from rbconfig.rb");
        let matched = arch_regex
            .captures(&line)
            .and_then(|c| c.name("arch"))
            .map(|s| s.as_str());
        match matched {
            Some(name) => return name.to_string(),
            _ => continue,
        }
    }

    panic!("could not extract arch from rbconfig.rb");
}

fn extract_ruby_library_config(ruby_checkout_path: &Path, arch: &String) -> RubyConfig {
    let config_h = ruby_checkout_path.join(format!(".ext/include/{}/ruby/config.h", arch));
    let f = File::open(config_h).unwrap_or_else(|_| panic!("cannot find config.h for {}", arch));
    let f = BufReader::new(f);
    let config = RubyConfig {
        libz: false,
        libcrypt: false,
    };
    f.lines().fold(config, |config, line| {
        let line = line.expect("could not read from config.h");
        if line.starts_with("#define HAVE_LIBZ 1") {
            RubyConfig {
                libz: true,
                ..config
            }
        } else if line.starts_with("#define HAVE_LIBCRYPT 1") {
            RubyConfig {
                libcrypt: true,
                ..config
            }
        } else {
            config
        }
    })
}

#[cfg(unix)]
fn make_configure(ruby_checkout_path: &Path) -> Output {
    let o = Command::new("autoreconf")
        .arg("--install")
        .current_dir(ruby_checkout_path)
        .status()?;
    check_process_success("autoreconf --install", o)
}

#[cfg(windows)]
fn make_configure(_: &Path) -> Output {
    Ok(())
}

#[cfg(unix)]
fn run_configure(ruby_checkout_path: &Path) -> Output {
    let mut command = Command::new("./configure");

    command.arg("--without-gmp").arg("--disable-jit-support");

    // This is gross, because it's very limited, but it is the simplest
    // thing we can do, and calling other build systems inside build systems
    // is destined to be gross anyway.
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    if env::var("CARGO_CFG_TARGET_ARCH")
        .map(|v| v == "aarch64")
        .unwrap_or(false)
    {
        command
            .arg("--target=aarch64-unknown-linux-gnu")
            .arg("--host=x86_64")
            .env("CC", "aarch64-linux-gnu-gcc")
            .env("AR", "aarch64-linux-gnu-ar")
            .env("RANLIB", "aarch64-linux-gnu-ranlib");
    }

    let o = command.current_dir(ruby_checkout_path).status()?;
    check_process_success("./configure", o)
}

#[cfg(windows)]
fn run_configure(ruby_checkout_path: &Path) -> Output {
    let mut command = Command::new(ruby_checkout_path.join("win32/configure.bat"));
    command
        .arg("--without-gmp")
        .arg("--disable-mjit-support")
        .arg("--with-static-linked-ext")
        .arg("--disable-install-doc")
        .arg("--with-ext=ripper")
        .envs(find_tool("nmake.exe")?.env().iter().cloned())
        .current_dir(ruby_checkout_path);
    #[cfg(target_arch = "x86_64")]
    command.arg("--target=x64-mswin64");
    let o = command.status()?;
    check_process_success("win32/configure.bat", o)
}

#[cfg(unix)]
fn build_ruby(ruby_checkout_path: &Path) -> Output {
    let o = Command::new("make")
        .arg("-j")
        .arg("main")
        .current_dir(ruby_checkout_path)
        .status()?;
    check_process_success("make", o)
}

#[cfg(windows)]
fn build_ruby(ruby_checkout_path: &Path) -> Output {
    let o = find_tool("nmake.exe")?
        .to_command()
        .arg("main")
        .current_dir(ruby_checkout_path)
        .status()?;
    check_process_success("nmake", o)
}

#[cfg(windows)]
fn find_tool(tool: &str) -> Result<cc::Tool, Box<dyn Error>> {
    let target = env::var("TARGET")?;
    cc::windows_registry::find_tool(&target, tool)
        .ok_or_else(|| format!("Failed to find {}", tool).into())
}

fn check_process_success(command: &str, code: ExitStatus) -> Output {
    if code.success() {
        Ok(())
    } else {
        Err(format!("Command {} failed with: {}", command, code).into())
    }
}

fn get_ruby_checkout_sha() -> String {
    String::from_utf8(
        Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir("./ruby_checkout")
            .output()
            .expect("git rev-parse shouldn't fail")
            .stdout,
    )
    .expect("output should be valid utf8")
}
