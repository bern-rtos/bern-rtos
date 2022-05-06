#![allow(unused)]

use crate::TaskInfo;

#[cfg(feature = "trace_impl")]
extern "Rust" {
    fn _rtos_trace_task_new(id: u32);
    fn _rtos_trace_task_terminate(id: u32);
    fn _rtos_trace_task_exec_begin(id: u32);
    fn _rtos_trace_task_exec_end();
    fn _rtos_trace_task_ready_begin(id: u32);
    fn _rtos_trace_task_ready_end(id: u32);

    fn _rtos_trace_system_idle();

    fn _rtos_trace_isr_enter();
    fn _rtos_trace_isr_exit();
    fn _rtos_trace_isr_exit_to_scheduler();

    fn _rtos_trace_marker(id: u32);
    fn _rtos_trace_marker_begin(id: u32);
    fn _rtos_trace_marker_end(id: u32);

    fn _rtos_trace_task_send_info(id: u32, info: TaskInfo);
    fn _rtos_trace_time() -> u64;

    fn _rtos_trace_system_description();
    fn _rtos_trace_sysclock() -> u32;
}

#[inline]
pub fn task_new(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_new(id) }
}
#[inline]
pub fn task_terminate(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_terminate(id) }
}
#[inline]
pub fn task_exec_begin(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_exec_begin(id) }
}
#[inline]
pub fn task_exec_end() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_exec_end() }
}
#[inline]
pub fn task_ready_begin(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_ready_begin(id) }
}
#[inline]
pub fn task_ready_end(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_ready_end(id) }
}

#[inline]
pub fn system_idle() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_system_idle() }
}

#[inline]
pub fn isr_enter() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_isr_enter() }
}
#[inline]
pub fn isr_exit() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_isr_exit() }
}
#[inline]
pub fn isr_exit_to_scheduler() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_isr_exit_to_scheduler() }
}

#[inline]
pub fn marker(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_marker(id) }
}
#[inline]
pub fn marker_begin(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_marker_begin(id) }
}
#[inline]
pub fn marker_end(id: u32) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_marker_end(id) }
}

#[inline]
pub fn task_send_info(id: u32, info: TaskInfo) {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_task_send_info(id, info) }
}
#[inline]
pub fn time() -> u64 {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_time() }
    #[cfg(not(feature = "trace_impl"))]
    0
}

#[inline]
pub fn system_description() {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_system_description() }
}
#[inline]
pub fn sysclock() -> u32 {
    #[cfg(feature = "trace_impl")]
    unsafe { _rtos_trace_sysclock() }
    #[cfg(not(feature = "trace_impl"))]
    0
}