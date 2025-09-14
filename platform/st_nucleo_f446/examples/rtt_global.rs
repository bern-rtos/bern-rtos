//! Very basic global logger for the `Log` facade using RTT as backend.

use log::{Level, Log, Metadata, Record};
use rtt_target::rprintln;

macro_rules! color {
    (r, $string:expr) => {
        concat!("\x1B[2;31m", $string, "\x1B[0m")
    }; // red, regular font
    (g, $string:expr) => {
        concat!("\x1B[2;32m", $string, "\x1B[0m")
    }; // green, regular font
    (y, $string:expr) => {
        concat!("\x1B[2;33m", $string, "\x1B[0m")
    }; // yellow, regular font
    (b, $string:expr) => {
        concat!("\x1B[2;34m", $string, "\x1B[0m")
    }; // blue, regular font
    (m, $string:expr) => {
        concat!("\x1B[2;35m", $string, "\x1B[0m")
    }; // magenta, regular font
    (c, $string:expr) => {
        concat!("\x1B[2;36m", $string, "\x1B[0m")
    }; // cyan, regular font
    (gr, $string:expr) => {
        concat!("\x1B[2;90m", $string, "\x1B[0m")
    }; // grey, regular font
    (r, b, $string:expr) => {
        concat!("\x1B[1;31m", $string, "\x1B[0m")
    }; // red, bold font
    (g, b, $string:expr) => {
        concat!("\x1B[1;32m", $string, "\x1B[0m")
    }; // green, bold font
    (y, b, $string:expr) => {
        concat!("\x1B[1;33m", $string, "\x1B[0m")
    }; // yellow, bold font
    (b, b, $string:expr) => {
        concat!("\x1B[1;34m", $string, "\x1B[0m")
    }; // blue, bold font
    (m, b, $string:expr) => {
        concat!("\x1B[1;35m", $string, "\x1B[0m")
    }; // magenta, bold font
    (c, b, $string:expr) => {
        concat!("\x1B[1;36m", $string, "\x1B[0m")
    }; // cyan, bold font
    (gr, b, $string:expr) => {
        concat!("\x1B[1;90m", $string, "\x1B[0m")
    }; // cyan, bold font
}

pub struct RttLogger;

impl RttLogger {
    pub const fn new() -> RttLogger {
        RttLogger
    }

    pub fn init(&self) {
        rtt_target::rtt_init_print!();
    }
}

impl Log for RttLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        match record.level() {
            Level::Error => rprintln!(
                concat!(color!(r, b, "{}"), " {} ({})"),
                record.level().as_str(),
                record.args(),
                record.module_path().unwrap_or("")
            ),
            Level::Warn => rprintln!(
                concat!(color!(y, b, "{}"), " {} ({})"),
                record.level().as_str(),
                record.args(),
                record.module_path().unwrap_or("")
            ),
            Level::Info => rprintln!(
                concat!(color!(g, b, "{}"), " {} ({})"),
                record.level().as_str(),
                record.args(),
                record.module_path().unwrap_or("")
            ),
            Level::Debug => rprintln!(
                concat!(color!(b, b, "{}"), " {} ({})"),
                record.level().as_str(),
                record.args(),
                record.module_path().unwrap_or("")
            ),
            Level::Trace => rprintln!(
                concat!(color!(gr, b, "{}"), " {} ({})"),
                record.level().as_str(),
                record.args(),
                record.module_path().unwrap_or("")
            ),
        }
    }

    fn flush(&self) {
        // nothing to do
    }
}
