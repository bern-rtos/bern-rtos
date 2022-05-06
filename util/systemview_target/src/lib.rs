#![no_std]

mod wrapper;
pub mod log;
mod macros;

use core::ptr::null;
use wrapper::*;
pub use rtos_trace::RtosTrace;
use rtos_trace::TaskInfo;

pub struct SystemView;

impl SystemView {
    pub fn init() {
        unsafe {
            SEGGER_SYSVIEW_Conf();
        }
    }

    pub fn send_system_description(desc: &str) {
        unsafe {
            SEGGER_SYSVIEW_SendSysDesc(desc.as_ptr());
        }
    }
}


impl RtosTrace for SystemView {
    fn task_new(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskCreate(id);
        }
    }

    fn task_send_info(id: u32, info: TaskInfo) {
        let name = if info.name.is_empty() {
            null()
        } else {
            info.name.as_ptr()
        };
        let info = SEGGER_SYSVIEW_TASKINFO {
            TaskID: id,
            sName: name,
            Prio: info.priority,
            StackBase: info.stack_base as u32,
            StackSize: info.stack_size as u32,
        };
        unsafe {
            SEGGER_SYSVIEW_SendTaskInfo(&info);
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

    fn task_ready_end(id: u32) {
        unsafe {
            SEGGER_SYSVIEW_OnTaskStopReady(id, 0);
        }
    }

    fn system_idle() {
        unsafe {
            SEGGER_SYSVIEW_OnIdle();
        }
    }

    fn isr_enter() {
        unsafe {
            SEGGER_SYSVIEW_RecordEnterISR();
        }
    }

    fn isr_exit() {
        unsafe {
            SEGGER_SYSVIEW_RecordExitISR();
        }
    }

    fn isr_exit_to_scheduler() {
        unsafe {
            SEGGER_SYSVIEW_RecordExitISRToScheduler();
        }
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