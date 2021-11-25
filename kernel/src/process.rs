use core::alloc::Layout;
use core::cell::Cell;
use core::ptr::NonNull;
use crate::mem::allocator::{Allocator, AllocError};
use crate::mem::bump_allocator::BumpAllocator;
use crate::mem::Size;
use crate::task;
use bern_arch::arch::Arch;
use bern_arch::IStartup;
use bern_arch::startup::Region;

pub struct ProcessMemory {
    pub size: usize,

    pub data_start: *const u8,
    pub data_end: *const u8,
    pub data_load: *const u8,

    pub heap_start: *const u8,
    pub heap_end: *const u8,
}

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

        Arch::init_static_region(Region {
            start: self.memory.data_start as *const _,
            end: self.memory.data_end as *const _,
            data: Some(self.memory.data_load as *const _)
        });

        self.init.set(false);
    }

    pub fn create_thread(&'static self) -> task::TaskBuilder {
        self.lazy_init();
        task::Task::new(self)
    }

    pub(crate) fn request_memory(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.proc_allocator.allocate(layout)
    }

    pub(crate) fn start_addr(&self) -> *const u8 {
        self.memory.data_start
    }

    pub(crate) fn size(&self) -> Size {
        unsafe { Size::from_bytes(self.memory.size) }
    }
}

// Note(unsafe): The values of `Process` are read only.
unsafe impl Sync for Process { }