use core::ops::Deref;
use core::ptr::NonNull;
use crate::alloc::allocator::Allocator;
use crate::alloc::bump::Bump;
use bern_arch::arch::Arch;
use bern_arch::IStartup;
use bern_arch::startup::Region;
use bern_units::memory_size::Byte;
use crate::kernel::KERNEL;
use crate::mem::boxed::Box;
use crate::mem::linked_list::Node;
use crate::trace;

#[cfg(feature = "log-defmt")]
use defmt::Formatter;

#[cfg(feature = "log-rtt")]
use core::fmt::Display;


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
            })
        }
    }

    pub fn init<F>(&'static self, f: F) -> Result<(), ProcessError>
        where F: FnOnce(&Context)
    {
        if KERNEL.is_process_registered(self.inner.deref()) {
            return Err(ProcessError::AlreadyInit);
        }
        // Note(unsafe): Process is not initialized more than once.
        unsafe { self.inner.init_memory(); }
        KERNEL.register_process(self.node());

        KERNEL.start_init_process(self.inner.deref());

        f(&Context {
            process: self.inner.deref()
        });

        KERNEL.end_init_process();

        Ok(())
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
}

#[derive(Debug)]
pub enum ProcessError {
    NotInit,
    AlreadyInit,
    KernelAlreadyRunning,
}

impl ProcessInternal {
    ///
    /// # Safety
    /// Only call this method once.
    unsafe fn init_memory(&self) {
        trace!("Process memory: data 0x{:08X} - 0x{:08X}, alloc 0x{:08X} - 0x{:08X}",
            self.memory.data_start as usize,
            self.memory.data_end as usize,
            self.memory.heap_start as usize,
            self.memory.heap_end as usize
        );

        Arch::init_static_region(Region {
            start: self.memory.data_start as *const _,
            end: self.memory.data_end as *const _,
            data: Some(self.memory.data_load as *const _)
        });
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
        defmt::write!(fmt, "None    {:05}B/{:05}B ({}%)",
                      self.proc_allocator.usage().0,
                      self.proc_allocator.capacity().0,
                      (self.proc_allocator.usage().0 as f32 / self.proc_allocator.capacity().0 as f32 * 100f32) as u8
        )
    }
}

#[cfg(feature = "log-rtt")]
impl Display for ProcessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProcessError::NotInit => write!(f, "Kernel not initialized."),
            ProcessError::AlreadyInit => write!(f, "Kernel already initialized."),
            ProcessError::KernelAlreadyRunning => write!(f, "Kernel already running."),
        }
    }
}

#[cfg(feature = "log-rtt")]
impl Display for ProcessInternal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "None    {:05}B/{:05}B ({}%)",
               self.proc_allocator.usage().0,
               self.proc_allocator.capacity().0,
               (self.proc_allocator.usage().0 as f32 / self.proc_allocator.capacity().0 as f32 * 100f32) as u8
        )
    }
}