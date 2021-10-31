use std::env;

pub fn configure() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
            println!("cargo:rustc-link-arg-bins=/SAFESEH:NO")
        }
    }
    else {
        println!("cargo:rustc-link-arg-bins=-rdynamic")
    }
}
