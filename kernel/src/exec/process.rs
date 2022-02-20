use core::cell::Cell;
use core::ptr::NonNull;
use crate::alloc::allocator::Allocator;
use crate::alloc::bump::Bump;
use bern_arch::arch::Arch;
use bern_arch::IStartup;
use bern_arch::startup::Region;
use bern_base_types::memory_size::Byte;
use crate::kernel::{KERNEL, State};

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
    proc_allocator: Bump,
    init: Cell<bool>,
}

pub enum ProcessError {
    NotInit,
    AlreadyInit,
    KernelAlreadyRunning,
}

impl Process {
    pub const fn new(memory: ProcessMemory) -> Self {
        let proc_allocator = unsafe {
            Bump::new(
                NonNull::new_unchecked(memory.heap_start as *mut _),
                NonNull::new_unchecked(memory.heap_end as *mut _)
            )};

        Process {
            memory,
            proc_allocator,
            init: Cell::new(false),
        }
    }

    fn startup(&self) -> Result<(), ProcessError> {
        if self.init.get() {
            return Err(ProcessError::AlreadyInit);
        }

        if KERNEL.state() == State::Running {
            return Err(ProcessError::KernelAlreadyRunning);
        }

        Arch::init_static_region(Region {
            start: self.memory.data_start as *const _,
            end: self.memory.data_end as *const _,
            data: Some(self.memory.data_load as *const _)
        });

        self.init.set(false);
        return Ok(());
    }

    pub fn init<F>(&'static self, f: F) -> Result<(), ProcessError>
        where F: FnOnce(&Context)
    {
        self.startup()?;
        KERNEL.start_init_process(&self);

        f(&Context {
            process: self
        });

        KERNEL.end_init_process();
        return Ok(());
    }

    pub(crate) fn allocator(&self) -> &dyn Allocator {
        &self.proc_allocator
    }

    pub(crate) fn start_addr(&self) -> *const u8 {
        self.memory.data_start
    }

    pub(crate) fn size(&self) -> Byte {
        Byte(self.memory.size as u32)
    }
}

// Note(unsafe): The values of `Process` are read only.
unsafe impl Sync for Process { }

pub struct Context {
    process: &'static Process,
}

impl Context {
    pub(crate) fn process(&self) -> &'static Process {
        self.process
    }
}