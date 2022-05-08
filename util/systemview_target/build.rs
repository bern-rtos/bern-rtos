use std::env;
use std::path::PathBuf;

fn main() {
    // Create SystemView bindings
    println!("cargo:rerun-if-changed=src/wrapper.h");
    let bindings = bindgen::Builder::default()
        // prefix `cty` insteand of `std` for `no_std`
        .ctypes_prefix("cty")
        .use_core()
        .header("src/wrapper.h")
        .clang_arg("-Ilib/Config")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    #[cfg(not(feature = "callbacks-os-time"))]
    let os_time = "_DUMMY";
    #[cfg(feature = "callbacks-os-time")]
    let os_time = "CALLBACKS_OS_TIME";

    // Compile SystemView
    cc::Build::new()
        .file("lib/SEGGER/SEGGER_SYSVIEW.c")
        .file("lib/SEGGER/SEGGER_RTT.c")
        .file("lib/SEGGER/SEGGER_RTT_ASM_ARMv7M.S")
        .file("lib/impl/systemview_conf_cortex_m.c")
        .include("lib/SEGGER")
        .include("lib/Config")
        .define(os_time, "")
        .compile("systemview");
}