fn main() {
    // this is wild
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=foundation");
        println!("cargo:rustc-link-lib=framework=security");
    }

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=crypt");
}
