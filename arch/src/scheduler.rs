//! Scheduler.

/// Scheduler.
///
/// # Implementation
/// In addition to the trait the hardware context switch exception must be
/// implemented. The exception must
/// 1. Push the CPU registers to the stack
/// 2. Call `switch_context` from the kernel
///     - in: current task stack pointer in first parameter register (e.g. `r0`)
///     - out: next task stack pointer in return register (e.g. `r0`)
/// 3. Pop CPU register from new stack
///
/// On a ARM Cortex-M4 for example (simplified):
/// ```no_run
/// #[no_mangle]
/// #[naked]
/// extern "C" fn PendSV() {
///     unsafe {
///         asm!(
///         "push    {{lr}}",
///         "mrs     r0, psp",         // get process stack pointer
///         "stmdb   r0!, {{r4-r11}}", // push registers to stack A
///         "bl      switch_context",  // call kernel for context switch
///         "pop     {{lr}}",
///         "mov     r3, #3",
///         "msr     control, r3",      // run in unprivileged mode
///         "isb",
///         "ldmia   r0!, {{r4-r11}}",  // pop registers from stack B
///         "msr     psp, r0",          // set process stack pointer
///         "bx      lr",
///         options(noreturn),
///         )
///     }
/// }
/// ```
///
/// The exception can call `check_stack` to check wheter the there is enough
/// space left to store the registers.
///     - in: stack pointer after stacking
///     - out: 0 - stack would overflow, 1 - enought space left
pub trait IScheduler {
    /// Init the stack of task.
    ///
    /// # Safety
    /// The stack must be large enough for the initial stack frame.
    unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize;
    /// Start the first task.
    fn start_first_task(stack_ptr: *const usize) -> !;
    /// Trigger context switch exception.
    fn trigger_context_switch();
}