use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());
    File::create(out.join("bern_user.x"))
        .unwrap();
    println!("cargo:rerun-if-changed=bern_user.x");
}