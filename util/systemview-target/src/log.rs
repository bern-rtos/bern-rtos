use core::fmt::Write;
use crate::wrapper::*;


fn print(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Print(str.as_ptr());
    }
}

fn warn(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Warn(str.as_ptr());
    }
}

fn error(str: &str) {
    unsafe {
        SEGGER_SYSVIEW_Error(str.as_ptr());
    }
}


pub struct InfoWriter;
impl InfoWriter {
    pub fn get() -> InfoWriter {
        InfoWriter
    }
}
impl Write for InfoWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s);
        Ok(())
    }
}
#[macro_export]
macro_rules! info {
    () => {
        $crate::log::InfoWriter::get().write_str("\r\n\r\n").ok();
    };
    ($fmt:expr) => {
        $crate::log::InfoWriter::get().write_str(concat!($fmt, "\r\n\r\n")).ok();
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log::InfoWriter::get().write_fmt(format_args!(concat!($fmt, "\r\n\r\n"), $($arg)*)).ok();
    };
}


pub struct WarnWriter;
impl WarnWriter {
    pub fn get() -> WarnWriter {
        WarnWriter
    }
}
impl Write for WarnWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        warn(s);
        Ok(())
    }
}
#[macro_export]
macro_rules! warn {
    () => {
        $crate::log::WarnWriter::get().write_str("\r\n\r\n").ok();
    };
    ($fmt:expr) => {
        $crate::log::WarnWriter::get().write_str(concat!($fmt, "\r\n\r\n")).ok();
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log::WarnWriter::get().write_fmt(format_args!(concat!($fmt, "\r\n\r\n"), $($arg)*)).ok();
    };
}


pub struct ErrorWriter;
impl ErrorWriter {
    pub fn get() -> ErrorWriter {
        ErrorWriter
    }
}
impl Write for ErrorWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        error(s);
        Ok(())
    }
}
#[macro_export]
macro_rules! error {
    () => {
        $crate::log::ErrorWriter::get().write_str("\r\n\r\n").ok();
    };
    ($fmt:expr) => {
        $crate::log::ErrorWriter::get().write_str(concat!($fmt, "\r\n\r\n")).ok();
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::log::ErrorWriter::get().write_fmt(format_args!(concat!($fmt, "\r\n\r\n"), $($arg)*)).ok();
    };
}