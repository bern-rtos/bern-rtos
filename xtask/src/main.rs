use std::{
    env,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use duct::cmd;

//const CARGO_TARGET: &str = "thumbv7em-none-eabihf";
const CARGO_TARGET: &str = "thumbv7em-none-eabihf";
const CRATE_NAME: &str = "nucleo_f446_dev";
//const OPENOCD_INTERFACE: &str = "jlink";
const OPENOCD_INTERFACE: &str = "stlink";
const OPENOCD_TARGET: &str = "stm32f4x";
const RTT_TCP_PORT: u16 = 8765;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    env::set_current_dir(repo_root()?)?;

    match &args[..] {
        ["gdb"] => gdb()?,
        ["build"] => build()?,
        _ => println!("Cargo workflows

USAGE:
    cargo xtask [COMMAND]

COMMANDS:
    gdb     spawns a GDB server; flashes and runs firmware; prints logs
    build   builds project
"),
    }

    Ok(())
}

fn repo_root() -> Result<PathBuf, anyhow::Error> {
    // path to this crate (the directory that contains this crate's Cargo.toml)
    Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        // from there go one level up
        .parent()
        .unwrap()
        .to_owned())
}

fn build() -> Result<(), anyhow::Error> {
    cmd!(
        "cargo", "build",
        "--target", CARGO_TARGET,
        "--example", CRATE_NAME
    ).run()?;
    Ok(())
}

fn gdb() -> Result<(), anyhow::Error> {
    const BP_LENGTH: u8 = 2; // breakpoint length
    const RTT_BLOCK_IF_FULL: u32 = 2; // bit in `flags` field
    const RTT_FLAGS: u32 = 44; // offset of `flags` field in control block
    const RTT_ID: &str = "SEGGER RTT"; // control block ID
    const RTT_SIZE: u8 = 48; // control block size
    const THUMB_BIT: u32 = 1;

    build()?;

    let elf = Path::new("target")
        .join(CARGO_TARGET)
        .join("debug")
        .join("examples")
        .join(CRATE_NAME);

    // get symbol addresses from ELF
    let nm = cmd!("nm", "-C", &elf).read()?;
    let mut rtt = None;
    let mut main = None;
    for line in nm.lines() {
        if line.ends_with("_SEGGER_RTT") {
            rtt = line.splitn(2, ' ').next();
        } else if line.ends_with("main") {
            main = line.splitn(2, ' ').next();
        }
    }

    let rtt = u32::from_str_radix(
        rtt.ok_or_else(|| anyhow!("RTT control block not found"))?,
        16,
    )?;
    let main = u32::from_str_radix(
        main.ok_or_else(|| anyhow!("`main` function not found"))?,
        16,
    )? & !THUMB_BIT;

    #[rustfmt::skip]
    let openocd = cmd!(
        "openocd",
        "-d0",
        "-c", format!("source [find interface/{}.cfg]", OPENOCD_INTERFACE),
        //"-c", "jlink serial 269401093",
        "-c", "transport select hla_swd",
        "-c", format!("source [find target/{}.cfg]", OPENOCD_TARGET),
        "-c", "init",
        "-c", format!("rtt server start {} 0", RTT_TCP_PORT),
        "-c", "reset init",
        "-c", format!("flash write_image erase {}", elf.display()),
        "-c", "reset halt",
        "-c", format!("rtt setup {} {} {:?}", rtt, RTT_SIZE, RTT_ID),
        "-c", format!("bp {} {} hw", main, BP_LENGTH),
        "-c", "resume",
        "-c", format!("mww {} {}", rtt + RTT_FLAGS, RTT_BLOCK_IF_FULL),
        "-c", "rtt start",
    )
    .stderr_to_stdout()
    .reader()?;

    let mut lines = BufReader::new(openocd).lines();

    while let Some(line) = lines.next() {
        let line = line?;
        println!("{}", line);

        if line.contains("wrote") {
            println!("=> GDB server is ready. Attach debugger now.");
            break;
        }
    }

    cmd!("nc", "localhost", RTT_TCP_PORT.to_string())
        .pipe(cmd!("defmt-print", "-e", &elf))
        .run()?;

    // close `openocd` *after* `nc`
    drop(lines);

    Ok(())
}
