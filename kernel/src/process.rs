use core::alloc::Layout;
use core::ptr::NonNull;
use crate::mem::allocator::{Allocator, AllocError};
use crate::mem::bump_allocator::BumpAllocator;
use crate::stack::Stack;
use crate::task;

pub struct Process {
    proc_memory: &'static mut [u8],
    proc_allocator: BumpAllocator,
}

impl Process {
    pub fn new(proc_memory: &'static mut [u8]) -> Self {
        let proc_allocator = unsafe {
            BumpAllocator::new(
                NonNull::new_unchecked(proc_memory.as_mut_ptr()),
                proc_memory.len()
            )};

        Process {
            proc_memory,
            proc_allocator
        }
    }

    pub fn create_thread(&mut self) -> task::TaskBuilder {
        task::Task::new(self)
    }

    pub(crate) fn request_memory(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.proc_allocator.allocate(layout)
    }
}


#[macro_export]
macro_rules! alloc_process_memory {
    ($name:ident, $size:tt) => {
        {
            #[link_section = ".proc.$name"]
            static mut MEMORY: $crate::stack::Aligned<$crate::bern_arch::alignment_from_size!($size), [u8; $size]> =
                $crate::stack::Aligned([0; $size]);

            // this is unsound, because the same stack can 'allocated' multiple times
            unsafe { &mut *MEMORY }
        }
    };
}