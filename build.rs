//! A build script to pass the installation prefix to the Rust compiler.
//!
//! This script checks for the `INSTALL_PREFIX` environment variable at compile
//! time. If the variable is set, its value is used as the installation prefix.
//! Otherwise, it defaults to `/usr/local`. This allows for flexible installation
//! paths, especially for packaging on Linux systems.
//!
//! The prefix is then exposed to the application code via the `INSTALL_PREFIX`
//! environment variable, which can be accessed using `option_env!`.

fn main() {
    // Allow overriding install prefix at compile-time via environment
    // Example: INSTALL_PREFIX=/usr cargo build --release
    let prefix = std::env::var("INSTALL_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
    println!("cargo:rustc-env=INSTALL_PREFIX={prefix}");
}
