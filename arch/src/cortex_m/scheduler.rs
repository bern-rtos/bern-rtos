//! ARM Cortex-M implementation of [`IScheduler`] and context switch.

use core::mem;
use cortex_m::peripheral::SCB;

use crate::scheduler::IScheduler;
use crate::arch::Arch;
use crate::arch::register::{StackFrame, StackFrameExtension};

/// Pendable service call.
///
/// Storing and loading registers in context switch.
///
/// Exception is triggered by `cortex_m::peripheral::SCB::PendSV()`.
#[no_mangle]
#[naked] // todo: move to separate assembly file and introduce at link time
extern "C" fn PendSV() {
    // Source: Definitive Guide to Cortex-M3/4, p. 342
    // store stack of current task
    unsafe {
        asm!(
        "push    {{lr}}",
        "mrs     r3, psp",
        "sub     r0, r3, #32",  // psp after register push
        "bl      check_stack",  // in: psp (r0), out: store context (1), stack would overflow (0)
        "cmp     r0, #1",       // stack valid?
        "it      eq",
        "stmdbeq r3!, {{r4-r11}}",
        "mov     r0, r3",
        "bl      switch_context",
        "pop     {{lr}}",
        "mov     r3, #3",        // todo: read from function
        "msr     control, r3",   // switch to unprivileged thread mode
        "isb",
        "ldmia   r0!, {{r4-r11}}",
        "msr     psp, r0",
        "bx      lr",
        options(noreturn),
        )
    }
}

impl IScheduler for Arch {
    unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize {
        let stack_frame_offset = mem::size_of::<StackFrame>() / mem::size_of::<usize>();
        let mut stack_frame: &mut StackFrame =
            mem::transmute(&mut *stack_ptr.offset(-(stack_frame_offset as isize)));
        stack_frame.r0 = arg as u32;
        stack_frame.lr = exit as u32;
        stack_frame.pc = entry as u32;
        stack_frame.xpsr = 0x01000000; // todo: document

        let stack_ptr_offset =
            (mem::size_of::<StackFrame>() + mem::size_of::<StackFrameExtension>()) /
                mem::size_of::<usize>();
        stack_ptr.offset(-(stack_ptr_offset as isize))
    }

    fn start_first_task(stack_ptr: *const usize) -> ! {
        unsafe {
            asm!(
            "msr   psp, {1}",       // set process stack pointer -> task stack
            "msr   control, {0}",   // switch to thread mode
            "isb",                  // recommended by ARM
            "pop   {{r4-r11}}",     // pop register we initialized
            "pop   {{r0-r3,r12,lr}}", // force function entry
            "pop   {{pc}}",         // 'jump' to the task entry function we put on the stack
            in(reg) 0x3,            // unprivileged task
            in(reg) stack_ptr as u32,
            options(noreturn),
            );
        }
    }

    #[inline]
    fn trigger_context_switch() {
        SCB::set_pendsv();
    }
}