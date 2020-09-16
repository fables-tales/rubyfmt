use std::io::{self, Write};
use std::process::Command;

fn main() {
    #[cfg(target_os = "linux")]
    let libname = "ruby-static";
    #[cfg(target_os = "macos")]
    let libname = "ruby.2.6-static";

    let path = std::env::current_dir().expect("is current");
    let ruby_checkout_path = path.join("ruby_checkout/ruby-2.6.6/");
    if !ruby_checkout_path
        .join(format!("lib{}.a", libname))
        .exists()
    {
        let o = Command::new("bash")
            .arg("-c")
            .arg(format!(
                "autoconf && {}/configure --without-gmp --disable-jit-support && make -j",
                ruby_checkout_path.display()
            ))
            .current_dir(&ruby_checkout_path)
            .output()
            .expect("works1 ");
        if !o.status.success() {
            io::stdout().write_all(&o.stdout).unwrap();
            io::stderr().write_all(&o.stderr).unwrap();
            panic!("failed subcommand");
        }
    }
    if !ruby_checkout_path.join("libripper.2.6-static.a").exists() {
        let o = Command::new("bash")
            .arg("-c")
            .arg("ar crus libripper.2.6-static.a ext/ripper/ripper.o")
            .current_dir(&ruby_checkout_path)
            .output()
            .expect("works");
        if !o.status.success() {
            panic!("failed subcommand");
        }
    }

    cc::Build::new()
        .file("src/rubyfmt.c")
        .include(format!("{}/include", ruby_checkout_path.display()))
        .include(format!(
            "{}/.ext/include/x86_64-darwin20",
            ruby_checkout_path.display()
        ))
        .include(format!(
            "{}/.ext/include/x86_64-darwin19",
            ruby_checkout_path.display()
        ))
        .include(format!(
            "{}/.ext/include/x86_64-darwin18",
            ruby_checkout_path.display()
        ))
        .include(format!(
            "{}/.ext/include/x86_64-linux",
            ruby_checkout_path.display()
        ))
        .compile("librubyfmt_c");

    println!(
        "cargo:rustc-link-search=native={}/ruby_checkout/ruby-2.6.6",
        path.display()
    );
    println!("cargo:rustc-link-lib=static={}", libname);
    println!("cargo:rustc-link-lib=static=ripper.2.6-static");
    println!("cargo:rustc-link-lib=dylib=z");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=crypt");
}
