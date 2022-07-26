#[cfg(windows)]
use std::env;
use std::error::Error;
use std::path::Path;
use std::process::{Command, ExitStatus};

type Output = Result<(), Box<dyn Error>>;

fn main() -> Output {
    #[cfg(target_os = "linux")]
    let libname = "ruby-static";
    #[cfg(target_os = "macos")]
    let libname = "ruby.2.7-static";
    #[cfg(all(target_arch = "x86_64", windows))]
    let libname = "x64-vcruntime140-ruby270-static";
    #[cfg(all(target_arch = "x86", windows))]
    let libname = "vcruntime140-ruby270-static";
    #[cfg(all(target_env = "gnu", windows))]
    compile_error!("rubyfmt on Windows is currently only supported with msvc");

    let path = std::env::current_dir()?;
    let ruby_checkout_path = path.join("ruby_checkout");

    let _ = Command::new("git")
        .args(&["submodule", "update", "--init"])
        .status();

    make_configure(&ruby_checkout_path)?;
    run_configure(&ruby_checkout_path)?;
    build_ruby(&ruby_checkout_path)?;
    #[cfg(unix)]
    let ripper = "ext/ripper/ripper.o";
    #[cfg(windows)]
    let ripper = "ext/ripper/ripper.obj";
    cc::Build::new()
        .file("src/rubyfmt.c")
        .object(ruby_checkout_path.join(&ripper))
        .include(ruby_checkout_path.join("include"))
        .include(ruby_checkout_path.join(".ext/include/arm64-darwin21"))
        .include(ruby_checkout_path.join(".ext/include/x86_64-darwin21"))
        .include(ruby_checkout_path.join(".ext/include/x86_64-darwin20"))
        .include(ruby_checkout_path.join(".ext/include/x86_64-darwin19"))
        .include(ruby_checkout_path.join(".ext/include/x86_64-darwin18"))
        .include(ruby_checkout_path.join(".ext/include/x86_64-linux"))
        .include(ruby_checkout_path.join(".ext/include/x64-mswin64_140"))
        .include(ruby_checkout_path.join(".ext/include/i386-mswin32_140"))
        .compile("rubyfmt_c");

    println!(
        "cargo:rustc-link-search=native={}",
        ruby_checkout_path.display()
    );
    println!("cargo:rustc-link-lib=static={}", libname);
    #[cfg(not(windows))]
    println!("cargo:rustc-link-lib=dylib=z");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=crypt");

    Ok(())
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
    let o = Command::new("./configure")
        .arg("--without-gmp")
        .arg("--disable-jit-support")
        .current_dir(ruby_checkout_path)
        .status()?;
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
        .current_dir(ruby_checkout_path)
        .status()?;
    check_process_success("make", o)
}

#[cfg(windows)]
fn build_ruby(ruby_checkout_path: &Path) -> Output {
    let o = find_tool("nmake.exe")?
        .to_command()
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
