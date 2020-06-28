use std::process::Command;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let path = std::env::current_dir().expect("is current");
    let ruby_checkout_path = format!("{}/ruby_checkout/ruby-2.6.6", path.display());
    if !Path::new(&format!("{}/libruby.2.6-static.a", ruby_checkout_path)).exists() {
        let o = Command::new("bash")
            .arg("-c")
            .arg(format!("{}/configure && make -j", ruby_checkout_path))
            .current_dir(&ruby_checkout_path)
            .output().expect("works1 ");
        if !o.status.success() {
            io::stdout().write_all(&o.stdout).unwrap();
            io::stderr().write_all(&o.stderr).unwrap();
            panic!("failed subcommand");
        }
    }
    if !Path::new(&format!("{}/ruby_checkout/ruby-2.6.6/libripper.2.6-static.a", path.display())).exists() {
        let o = Command::new("bash")
            .arg("-c")
            .arg("ar crus libripper.2.6-static.a ext/ripper/ripper.o")
            .current_dir(&ruby_checkout_path)
            .output().expect("works");
        if !o.status.success() {
            panic!("failed subcommand");
        }
    }
    cc::Build::new()
        .file("src/rubyfmt.c")
        .include(format!("{}/include", ruby_checkout_path))
        .include(format!("{}/.ext/include/x86_64-darwin19", ruby_checkout_path))
        .compile("librubyfmt_c");

    println!("cargo:rustc-link-search=native={}/ruby_checkout/ruby-2.6.6", path.display());
    println!("cargo:rustc-link-lib=static=ruby.2.6-static");
    println!("cargo:rustc-link-lib=static=ripper.2.6-static");
}
