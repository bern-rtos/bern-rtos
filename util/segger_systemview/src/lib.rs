#![no_std]

mod wrapper;
pub mod log;

use wrapper::*;

pub struct SystemView { }


impl SystemView {
    pub fn new() -> SystemView {
        unsafe {
            SEGGER_SYSVIEW_Conf();
        }

        SystemView {

        }
    }
}
