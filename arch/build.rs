
use std::path::PathBuf;
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Set cargo flags from target toolchain
    // Source: `cortex-m` crate
    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=arm_cortex_m");
        println!("cargo:rustc-cfg=armv6m");
    } else if target.starts_with("thumbv7m-") {
        println!("cargo:rustc-cfg=arm_cortex_m");
        println!("cargo:rustc-cfg=armv7m");
    } else if target.starts_with("thumbv7em-") {
        println!("cargo:rustc-cfg=arm_cortex_m");
        println!("cargo:rustc-cfg=armv7em");
    } else if target.starts_with("thumbv8m.base") {
        println!("cargo:rustc-cfg=arm_cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_base");
    } else if target.starts_with("thumbv8m.main") {
        println!("cargo:rustc-cfg=arm_cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_main");
    }

    if target.ends_with("-eabihf") {
        println!("cargo:rustc-cfg=has_fpu");
    }
}