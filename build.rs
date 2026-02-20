fn main() {
    if std::env::consts::OS == "macos" {
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=dylib=IOReport");
    }
}
