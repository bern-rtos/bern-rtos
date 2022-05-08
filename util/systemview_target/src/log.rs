use core::fmt;
use log::{Level, Log, Metadata, Record};
use crate::SystemView;
use crate::wrapper::*;
use fmt::Write;


pub fn print(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Print(str.as_ptr());
    }
}

pub fn warn(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Warn(str.as_ptr());
    }
}

pub fn error(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Error(str.as_ptr());
    }
}

#[macro_export]
macro_rules! info {
    () => {
        $crate::log::print("\0");
    };
    ($fmt:expr) => {
        $crate::log::print(concat!($fmt, "\0"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        let mut s: $crate::heapless::String<64> = $crate::heapless::String::new();
        write!(&mut s, $fmt, $($arg)*).ok();
        $crate::log::print(s.as_str());
    };
}

#[macro_export]
macro_rules! warn {
    () => {
        $crate::log::warn("\0");
    };
    ($fmt:expr) => {
        $crate::log::warn(concat!($fmt, "\0"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        let mut s: $crate::heapless::String<64> = $crate::heapless::String::new();
        write!(&mut s, $fmt, $($arg)*).ok();
        $crate::log::warn(s.as_str());
    };
}

#[macro_export]
macro_rules! error {
    () => {
        $crate::log::error("\0");
    };
    ($fmt:expr) => {
        $crate::log::error(concat!($fmt, "\0"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        let mut s: $crate::heapless::String<64> = $crate::heapless::String::new();
        write!(&mut s, $fmt, $($arg)*).ok();
        $crate::log::error(s.as_str());
    };
}


impl Log for SystemView {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut s: heapless::String<128> = heapless::String::new();
        write!(&mut s, "{} {}", record.level().as_str(), record.args()).ok();

        match record.metadata().level() {
            Level::Error => error(s.as_str()),
            Level::Warn => warn(s.as_str()),
            Level::Info | Level::Debug | Level::Trace => print(s.as_str()),
        }
    }

    fn flush(&self) {
        // nothing to do
    }
}