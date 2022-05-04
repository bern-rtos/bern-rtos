use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::ptr::{NonNull, null_mut};
use core::sync::atomic::{AtomicPtr, Ordering};
use bern_arch::arch::Arch;
use bern_arch::{IMemoryProtection, IStartup};
use bern_arch::memory_protection::{Access, Config, Permission, Type};
use bern_conf::CONF;
use crate::alloc::bump::Bump;
use crate::alloc::allocator::Allocator;
use crate::exec::process::{ProcessInternal};
use crate::{log, sched, trace};
use crate::mem::boxed::Box;
use crate::mem::linked_list::{LinkedList, Node};

#[cfg(feature = "log-defmt")]
use defmt::Formatter;

#[link_section = ".kernel"]
pub(crate) static KERNEL: Kernel = Kernel::new();

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Startup,
    Running,
}

pub struct Kernel {
    /// Kernel state.
    state: Cell<State>,
    /// Currently initializing process.
    init_process: AtomicPtr<usize>,
    /// Allocator for kernel modules.
    allocator: UnsafeCell<MaybeUninit<Bump>>,
    /// List of processes.
    processes: LinkedList<ProcessInternal>,
}

impl Kernel {
    pub(crate) const fn new() -> Self {
        Kernel {
            state: Cell::new(State::Startup),
            init_process: AtomicPtr::new(null_mut()),
            allocator: UnsafeCell::new(MaybeUninit::uninit()),
            processes: LinkedList::new(),
        }
    }

    pub(crate) fn init(&self) {
        setup_memory_regions();
        unsafe {
            let allocator = Bump::new(
                NonNull::new_unchecked(Arch::kernel_heap().start as *mut u8),
                NonNull::new_unchecked(Arch::kernel_heap().end as *mut u8));
            *self.allocator.get() = MaybeUninit::new(allocator);
        };

        sched::init();
    }

    pub(crate) fn start(&self) -> ! {
        crate::debug!("Staring kernel.");
        self.state.replace(State::Running);

        sched::start();
    }
    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    pub(crate) fn start_init_process(&self, process: &'static ProcessInternal) {
        self.init_process.store(process as *const _ as *mut _, Ordering::Relaxed);
        trace!("Set init process to 0x{:08X}", process as *const _ as usize);
    }
    pub(crate) fn end_init_process(&self) {
        self.init_process.store(null_mut(), Ordering::Release);
        trace!("Set init process to null");
    }
    pub(crate) fn process(&self) -> Option<&ProcessInternal> {
        let ptr = self.init_process.load(Ordering::Acquire) as *const ProcessInternal;
        unsafe { ptr.as_ref() }
    }

    pub(crate) fn allocator(&self) -> &Bump {
        unsafe { (*self.allocator.get()).assume_init_ref() }
    }

    pub(crate) fn register_process(&self, process_node: Box<Node<ProcessInternal>>) {
        self.processes.push_back(process_node);
    }

    pub(crate) fn is_process_registered(&self, process: &ProcessInternal) -> bool {
        for proc in self.processes.iter() {
            if proc.start_addr() == process.start_addr() {
                return true;
            }
        }
        false
    }
}


fn setup_memory_regions() {
    Arch::init_static_region(Arch::kernel_data());

    // Memory regions 0..2 are reserved for threads
    Arch::disable_memory_region(0);
    Arch::disable_memory_region(1);
    Arch::disable_memory_region(2);

    // Allow flash read/exec
    Arch::enable_memory_region(
        3,
        Config {
            addr: CONF.memory_map.flash.start_address as *const _,
            memory: Type::Flash,
            size: CONF.memory_map.flash.size,
            access: Access { user: Permission::ReadOnly, system: Permission::ReadOnly },
            executable: true
        });

    // Allow peripheral RW
    Arch::enable_memory_region(
        4,
        Config {
            addr: CONF.memory_map.peripheral.start_address as *const _,
            memory: Type::Peripheral,
            size: CONF.memory_map.peripheral.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    // Allow shared read/write
    Arch::enable_memory_region(
        5,
        Config {
            addr: CONF.memory_map.sram.start_address as *const _,
            memory: Type::SramInternal,
            size: CONF.shared.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    for i in 6..Arch::n_memory_regions() {
        Arch::disable_memory_region(i);
    }
}

// Note(unsafe): Values within `KERNEL` are only changed at startup, this
// guarantees non-reentrant/single thread operation.
unsafe impl Sync for Kernel { }


pub fn init() {
    KERNEL.init();
}

pub fn start() -> ! {
    KERNEL.start();
}


#[cfg(feature = "log-defmt")]
impl defmt::Format for State {
    fn format(&self, fmt: Formatter) {
        match self {
            State::Startup => defmt::write!(fmt, "Starting up"),
            State::Running => defmt::write!(fmt, "Running"),
        }
    }
}

pub(crate) fn print_stats() {
    log::info!("Kernel stats");
    log::info!("============");
    log::info!("Allocator: {}B/{}B ({}%)",
        KERNEL.allocator().usage().0,
        KERNEL.allocator().capacity().0,
        (KERNEL.allocator().usage().0 as f32 / KERNEL.allocator().capacity().0 as f32 * 100f32) as u8
    );

    log::info!("Process stats");
    log::info!("=============");
    log::info!("Name    Allocator");
    log::info!("----    ---------");
    for proc in KERNEL.processes.iter() {
        log::info!("{}", proc);
    }
}