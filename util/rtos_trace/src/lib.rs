#![no_std]

pub trait RtosTrace {
    fn task_new(id: u32);
    fn task_terminate(id: u32);
    fn task_exec_begin(id: u32);
    fn task_exec_end();
    fn task_ready_begin(id: u32);
    fn task_ready_end(id: u32);

    fn marker(id: u32);
    fn marker_begin(id: u32);
    fn marker_end(id: u32);
}