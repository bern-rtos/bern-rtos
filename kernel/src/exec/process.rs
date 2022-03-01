use core::cell::Cell;
use core::ops::Deref;
use core::ptr::NonNull;
use crate::alloc::allocator::Allocator;
use crate::alloc::bump::Bump;
use bern_arch::arch::Arch;
use bern_arch::IStartup;
use bern_arch::startup::Region;
use bern_units::memory_size::Byte;
use crate::kernel::{KERNEL, State};
use crate::log;
use crate::mem::boxed::Box;
use crate::mem::linked_list::Node;

#[cfg(feature = "log-defmt")]
use defmt::Formatter;

pub struct Process {
    inner: Node<ProcessInternal>,
}

impl Process {
    pub const fn new(memory: ProcessMemory) -> Self {
        let proc_allocator = unsafe {
            Bump::new(
                NonNull::new_unchecked(memory.heap_start as *mut _),
                NonNull::new_unchecked(memory.heap_end as *mut _)
            )};

        Process {
            inner: Node::new(ProcessInternal {
                memory,
                proc_allocator,
                init: Cell::new(false),
            })
        }
    }

    pub fn init<F>(&'static self, f: F) -> Result<(), ProcessError>
        where F: FnOnce(&Context)
    {
        KERNEL.register_process(self.node());

        match self.inner.startup() {
            Ok(_) => { },
            Err(e) => {
                log::warn!("Cannot init process: {}", e);
                return Err(e);
            }
        };

        KERNEL.start_init_process(self.inner.deref());

        f(&Context {
            process: self.inner.deref()
        });

        KERNEL.end_init_process();
        self.inner.init.set(true);

        return Ok(());
    }

    pub(crate) fn node(&self) -> Box<Node<ProcessInternal>> {
        unsafe {
            Box::from_raw(NonNull::new_unchecked(&self.inner as *const _ as *mut _))
        }
    }
}

unsafe impl Sync for Process { }


pub struct ProcessMemory {
    pub size: usize,

    pub data_start: *const u8,
    pub data_end: *const u8,
    pub data_load: *const u8,

    pub heap_start: *const u8,
    pub heap_end: *const u8,
}

pub struct ProcessInternal {
    memory: ProcessMemory,
    proc_allocator: Bump,
    init: Cell<bool>,
}

pub enum ProcessError {
    NotInit,
    AlreadyInit,
    KernelAlreadyRunning,
}

impl ProcessInternal {
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
unsafe impl Sync for ProcessInternal { }

pub struct Context {
    process: &'static ProcessInternal,
}

impl Context {
    pub(crate) fn process(&self) -> &'static ProcessInternal {
        self.process
    }
}

#[cfg(feature = "log-defmt")]
impl defmt::Format for ProcessError {
    fn format(&self, fmt: Formatter) {
        match self {
            ProcessError::NotInit => defmt::write!(fmt, "Kernel not initialized."),
            ProcessError::AlreadyInit => defmt::write!(fmt, "Kernel already initialized."),
            ProcessError::KernelAlreadyRunning => defmt::write!(fmt, "Kernel already running."),
        }
    }
}

#[cfg(feature = "log-defmt")]
impl defmt::Format for ProcessInternal {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "None    {}    {:05}B/{:05}B ({}%)",
                      self.init.get(),
                      self.proc_allocator.usage().0,
                      self.proc_allocator.capacity().0,
                      (self.proc_allocator.usage().0 as f32 / self.proc_allocator.capacity().0 as f32 * 100f32) as u8
        )
    }
}