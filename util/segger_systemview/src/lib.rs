#![no_std]

mod wrapper;
pub mod log;

use wrapper::*;
pub use rtos_trace::RtosTrace;

pub struct SystemView { }


impl SystemView {
    pub const fn new() -> SystemView {
        SystemView {

        }
    }

    pub fn init(&self) {
        unsafe {
            SEGGER_SYSVIEW_Conf();
        }
    }
}


impl RtosTrace for SystemView {
    fn task_new(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskCreate(id);
        }
    }

    fn task_terminate(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskTerminate(id);
        }
    }

    fn task_exec_begin(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskStartExec(id);
        }
    }

    fn task_exec_end() {
        unsafe {
            SEGGER_SYSVIEW_OnTaskStopExec();
        }
    }

    fn task_ready_begin(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskStartReady(id);
        }
    }

    fn task_ready_end(_id: u32) {
        //unsafe {
            //SEGGER_SYSVIEW_OnTaskStopReady(id, cause);
        //}
    }

    fn marker(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_Mark(id);
        }
    }

    fn marker_begin(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_MarkStart(id);
        }
    }

    fn marker_end(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_MarkStop(id);
        }
    }
}