//! ARM Cortex-M implementation of [`IScheduler`] and context switch.

use core::mem;
use cortex_m::peripheral::SCB;

use crate::scheduler::IScheduler;
use crate::arch::Arch;
use crate::arch::register::{StackFrame, StackFrameExtension, StackSettings};

/// Pendable service call.
///
/// Storing and loading registers in context switch.
///
/// Exception is triggered by `cortex_m::peripheral::SCB::PendSV()`.
#[no_mangle]
#[naked] // todo: move to separate assembly file and introduce at link time
extern "C" fn PendSV() {
    // Based on "Definitive Guide to Cortex-M3/4", p. 349
    unsafe {
        asm!(
        "mrs      r1, psp",

        "mov      r2, lr",
        "tst      r2, #10",         // was FPU used?
        "ite      eq",
        "subeq    r0, r1, #104",    // psp with FPU registers after register push
        "subne    r0, r1, #40",     // psp without FPU registers after register push
        "push     {{r1,r2}}",
        "bl       check_stack",     // in: psp (r0), out: store context (1), stack would overflow (0)
        "pop      {{r1,r2}}",
        "cmp      r0, #1",          // stack invalid?
        "itt      ne",              // if stack invalid
        "movne    r0, 0",           // set psp to 0, signal `switch_context` an error
        "bne      switch",
                                    // else store
        "tst      r2, #10",         // was FPU used?
        "it       eq",
        "vstmdbeq r1!, {{s16-s31}}", // push FPU registers
        "mrs      r3, control",
        "stmdb    r1!, {{r2-r11}}", // push LR, control and remaining registers
        "mov      r0, r1",

        "switch:  bl switch_context",

        "ldmia    r0!, {{r2-r11}}",
        "msr      control, r3",
        "isb",
        "mov      lr, r2",
        "tst      lr, #10",         // was FPU used?
        "it       eq",
        "vldmiaeq r0!, {{s16-s31}}", // pop FPU registers
        "msr      psp, r0",
        "bx       lr",
        options(noreturn),
        )
    }
}

impl IScheduler for Arch {
    unsafe fn init_task_stack(stack_ptr: *mut usize, entry: *const usize, arg: *const usize, exit: *const usize) -> *mut usize {
        let mut stack_offset = mem::size_of::<StackFrame>() / mem::size_of::<usize>();
        let mut stack_frame: &mut StackFrame =
            mem::transmute(&mut *stack_ptr.offset(-(stack_offset as isize)));
        stack_frame.r0 = arg as u32;
        stack_frame.lr = exit as u32;
        stack_frame.pc = entry as u32;
        stack_frame.xpsr = 0x01000000; // todo: document

        // we don't have to initialize r4-r11
        stack_offset += mem::size_of::<StackFrameExtension>() / mem::size_of::<usize>();

        stack_offset += mem::size_of::<StackSettings>() / mem::size_of::<usize>();
        let mut stack_settings: &mut StackSettings =
            mem::transmute(&mut *stack_ptr.offset(-(stack_offset as isize)));
        stack_settings.exception_lr = 0xFFFFFFFD;  // thread mode using psp
        stack_settings.control = 0x3; // unprivileged thread mode

        stack_ptr.offset(-(stack_offset as isize))
    }

    fn start_first_task(stack_ptr: *const usize) -> ! {
        unsafe {
            asm!(
            "ldmia r0!, {{r2,r3}}",
            "msr   psp, r0",           // set process stack pointer -> task stack
            "msr   control, r3",        // switch to thread mode
            "isb",
            "pop   {{r4-r11}}",
            "pop   {{r0-r3,r12,lr}}",   // force function entry
            "pop   {{pc}}",             // 'jump' to the task entry function we put on the stack
            in("r0") stack_ptr as u32,
            options(noreturn),
            );
        }
    }

    #[inline]
    fn trigger_context_switch() {
        SCB::set_pendsv();
    }
}