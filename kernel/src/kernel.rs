use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use bern_arch::arch::Arch;
use bern_arch::{IMemoryProtection, IStartup};
use bern_arch::memory_protection::{Access, Config, Permission, Type};
use bern_conf::CONF;
use bern_units::memory_size::ExtByte;
use crate::alloc::bump::Bump;
use crate::exec::process::Process;
use crate::sched;

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
    init_process: Cell<Option<&'static Process>>,
    /// Allocator for kernel modules.
    allocator: UnsafeCell<MaybeUninit<Bump>>,
}

impl Kernel {
    pub(crate) const fn new() -> Self {
        Kernel {
            state: Cell::new(State::Startup),
            init_process: Cell::new(None),
            allocator: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub(crate) fn init(&self) {
        unsafe {
            let allocator = Bump::new(
                NonNull::new_unchecked(Arch::kernel_heap().start as *mut u8),
                NonNull::new_unchecked(Arch::kernel_heap().end as *mut u8));
            *self.allocator.get() = MaybeUninit::new(allocator);
        };

        setup_memory_regions();
        sched::init();
    }

    pub(crate) fn start(&self) -> ! {
        self.state.replace(State::Running);

        sched::start();
    }
    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    pub(crate) fn start_init_process(&self, process: &'static Process) {
        self.init_process.replace(Some(process));
    }
    pub(crate) fn end_init_process(&self) {
        self.init_process.replace(None);
    }
    pub(crate) fn process(&self) -> Option<&Process> {
        self.init_process.get()
    }

    pub(crate) fn allocator(&self) -> &Bump {
        unsafe { (*self.allocator.get()).assume_init_ref() }
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
            addr: CONF.memory.flash.start_address as *const _,
            memory: Type::Flash,
            size: CONF.memory.flash.size,
            access: Access { user: Permission::ReadOnly, system: Permission::ReadOnly },
            executable: true
        });

    // Allow peripheral RW
    Arch::enable_memory_region(
        4,
        Config {
            addr: CONF.memory.peripheral.start_address as *const _,
            memory: Type::Peripheral,
            size: CONF.memory.peripheral.size,
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    // Allow .data & .bss read/write
    Arch::enable_memory_region(
        5,
        Config {
            addr: CONF.memory.sram.start_address as *const _,
            memory: Type::SramInternal,
            size: 4.kB().into(), // todo: read from linker symbol or config
            access: Access { user: Permission::ReadWrite, system: Permission::ReadWrite },
            executable: false
        });

    Arch::disable_memory_region(6);
    Arch::disable_memory_region(7);
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