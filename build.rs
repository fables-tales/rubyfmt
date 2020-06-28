fn main() {
    println!("cargo:rustc-link-lib=framework=foundation");
    println!("cargo:rustc-link-lib=dylib=gmp");
}
