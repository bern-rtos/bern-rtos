#![no_std]

mod macros;
pub mod trace;

pub struct  TaskInfo {
    pub name: &'static str,
    pub priority: u32,
    pub stack_base: usize,
    pub stack_size: usize,
}

pub trait RtosTrace {
    fn task_new(id: u32);
    fn task_send_info(id: u32, info: TaskInfo);
    fn task_terminate(id: u32);
    fn task_exec_begin(id: u32);
    fn task_exec_end();
    fn task_ready_begin(id: u32);
    fn task_ready_end(id: u32);

    fn system_idle();

    fn isr_enter();
    fn isr_exit();
    fn isr_exit_to_scheduler();

    fn marker(id: u32);
    fn marker_begin(id: u32);
    fn marker_end(id: u32);
}

pub trait RtosTraceOSCallbacks {
    fn task_list();
    /// Get system time in microseconds.
    fn time() -> u64;
}

pub trait RtosTraceApplicationCallbacks {
    fn system_description();
    fn sysclock() -> u32;
}