#![no_std]

mod macros;
pub mod trace;

pub trait RtosTrace {
    fn task_new(id: u32);
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
