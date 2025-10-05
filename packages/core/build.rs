#[allow(clippy::disallowed_macros)]
fn main() {
    // Link Security.framework on macOS
    // Note: println! is required in build scripts to communicate with Cargo
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-search=framework=/System/Library/Frameworks");
    }
}
