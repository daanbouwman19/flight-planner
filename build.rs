fn main() {
    // Allow overriding install prefix at compile-time via environment
    // Example: INSTALL_PREFIX=/usr cargo build --release
    let prefix = std::env::var("INSTALL_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
    println!("cargo:rustc-env=INSTALL_PREFIX={prefix}");
}
