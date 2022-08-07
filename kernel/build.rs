use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use const_format::formatcp;
use bern_conf::CONF;

const LINKER_SCRIPT_TEMPLATES: [(&str, &str); 6] = [
    ("${CONF_SHARED_SIZE}", &formatcp!("{}", CONF.shared.size.0)),
    ("${CONF_KERNEL_SIZE}", &formatcp!("{}", CONF.kernel.memory_size.0)),
    ("${CONF_PLACEMENT_SHARED}", CONF.data_placement.shared),
    ("${CONF_PLACEMENT_KERNEL}", CONF.data_placement.kernel),
    ("${CONF_PLACEMENT_PROCESSES}", CONF.data_placement.processes),
    ("${CONF_FLASH_NAME}", CONF.memory_map.flash.link_name),
];


fn main() {
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("bern.x"))
        .unwrap()
        .write_all(
            process_linker_script(String::from(include_str!("bern.x.in")))
                .as_bytes()
        )
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=bern.x");


    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(create_memory_x().as_bytes())
        .unwrap();
    println!("cargo:rerun-if-changed=memory.x");
}


fn process_linker_script(mut file: String) -> String {
    for (f, t) in LINKER_SCRIPT_TEMPLATES.iter() {
        file = file.replace(f, t);
    }
    file
}


fn create_memory_x() -> String {
    let mut file = format!("\
MEMORY {{
    {} : ORIGIN = 0x{:X}, LENGTH = {}
    {} : ORIGIN = 0x{:X}, LENGTH = {}",
        CONF.memory_map.flash.link_name,
        CONF.memory_map.flash.start_address,
        CONF.memory_map.flash.size.0,
        CONF.memory_map.sram.link_name,
        CONF.memory_map.sram.start_address,
        CONF.memory_map.sram.size.0,
    );

    for additional in CONF.memory_map.additional.iter() {
        file.push_str(format!("
    {} : ORIGIN = 0x{:X}, LENGTH = {}",
            additional.link_name,
            additional.start_address,
            additional.size.0
        ).as_str());
    }
    file.push_str("
}");

    file
}