use core::alloc::Layout;
use core::cell::Cell;
use core::ptr::NonNull;
use core::sync::atomic::AtomicBool;
use bern_common::process::ProcessMemory;
use crate::mem::allocator::{Allocator, AllocError};
use crate::mem::bump_allocator::BumpAllocator;
use crate::mem::Size;
use crate::stack::Stack;
use crate::task;

pub struct Process {
    memory: ProcessMemory,
    proc_allocator: BumpAllocator,
    init: Cell<bool>,
}

impl Process {
    pub const fn new(memory: ProcessMemory) -> Self {
        let proc_allocator = unsafe {
            BumpAllocator::new(
                NonNull::new_unchecked(memory.heap_start as *mut _),
                NonNull::new_unchecked(memory.heap_end as *mut _)
            )};

        Process {
            memory,
            proc_allocator,
            init: Cell::new(false),
        }
    }

    fn lazy_init(&self) {
        if self.init.get() {
            return;
        }

        /* todo: init process bss */

        self.init.set(false);
    }

    pub fn create_thread(&self) -> task::TaskBuilder {
        self.lazy_init();
        task::Task::new(self)
    }

    pub(crate) fn request_memory(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.proc_allocator.allocate(layout)
    }

    pub(crate) fn start_addr(&self) -> *const u8 {
        unsafe { self.memory.bss_start }
    }

    pub(crate) fn size(&self) -> Size {
        unsafe { Size::from_bytes(self.memory.size) }
    }
}

// Note(unsafe): The values of `Process` are read only.
unsafe impl Sync for Process { }