fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=foundation");
    }


    println!("cargo:rustc-link-lib=dylib=gmp");
}
