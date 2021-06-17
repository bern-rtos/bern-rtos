//! ARM Cortex-M implementation of [`ISyscall`].

use crate::arch::Arch;
use crate::syscall::ISyscall;

impl ISyscall for Arch {
    #[inline(always)]
    fn syscall(service: u8, arg0: usize, arg1: usize, arg2: usize) -> usize {
        // we need to move the arguments to the correct registers, because the
        // function is inlined
        let ret;
        unsafe { asm!(
            "push {{r4}}",
            "svc 0",
            "mov r0, r4",
            "pop {{r4}}",
            in("r0") service,
            in("r1") arg0,
            in("r2") arg1,
            in("r3") arg2,
            lateout("r0") ret,
        )}
        ret
    }
}

/// Extract and prepare system call for Rust handler.
/// r0 is used to store the service id, r1-r3 can contain call parameter.
///
/// The system call service id (`svc xy`) is not passed on, we have to
/// retrieve it from code memory. Thus we load the stack pointer from the
/// callee and read the link register. The link register is pointing to
/// instruction just after the system call, the system call service id is
/// placed two bytes before that.
///
/// The exception link register tells SVC which privilege mode the callee used
/// | EXC_RETURN (lr) | Privilege Mode     | Stack |
/// |-----------------|--------------------|------ |
/// | 0xFFFFFFF1      | Handler Mode       | MSP   |
/// | 0xFFFFFFF9      | Thread Mode        | MSP   |
/// | 0xFFFFFFFD      | Thread Mode        | PSP   |
/// | 0xFFFFFFE1      | Handler Mode (FPU) | MSP   |
/// | 0xFFFFFFE9      | Thread Mode (FPU)  | MSP   |
/// | 0xFFFFFFED      | Thread Mode (FPU)  | PSP   |
#[no_mangle]
#[naked]
unsafe extern "C" fn SVCall() {
    asm!(
    "push {{lr}}",
    "bl syscall_handler",
    "mov r4, r0", // let's use r4 as return value, because r0 is popped from stack
    "pop {{lr}}",
    "bx lr",
    options(noreturn),
    );
}