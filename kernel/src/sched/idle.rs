use core::sync::atomic::{compiler_fence, Ordering};
use crate::exec::process::{Process, ProcessMemory};
use crate::exec::thread::Thread;
use crate::stack::Stack;

extern "C" {
    static mut __smprocess_default_idle: usize;
    static mut __emprocess_default_idle: usize;
    static __siprocess_default_idle: usize;
}

#[no_mangle]
#[link_section = ".kernel.process"]
static BERN_DEFAULT_IDLE: Process = Process::new(
    unsafe { ProcessMemory {
        size: 256,

        data_start: (&__smprocess_default_idle) as *const _ as *const u8,
        data_end: (&__emprocess_default_idle) as *const _ as *const u8,
        data_load: (&__siprocess_default_idle) as *const _ as *const u8,

        heap_start: 1 as *const u8,
        heap_end: 1 as *const u8,
    }}
);

#[link_section = ".process.default_idle"]
static mut STACK: [u8; 256] = [0; 256];

pub(crate) fn init() {
    crate::trace!("Init idle thread");
    BERN_DEFAULT_IDLE.init(|c| {
        Thread::new(c)
            .idle_task()
            .stack(unsafe { Stack::new(&mut STACK, STACK.len()) })
            .spawn(|| {
                loop {
                    compiler_fence(Ordering::SeqCst);
                }
            });
    }).ok();
}