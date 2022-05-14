use core::cell::{Cell, UnsafeCell};
use core::mem::MaybeUninit;
use core::ptr::{NonNull, null_mut};
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use bern_arch::arch::Arch;
use bern_arch::{IMemoryProtection, IStartup};
use bern_arch::memory_protection::{Access, Config, Permission, Type};
use bern_conf::CONF;
use crate::alloc::bump::Bump;
use crate::alloc::allocator::{Allocator, AllocError};
use crate::exec::process::{ProcessInternal};
use crate::sched;
use crate::log::{error, info, debug, trace};
use crate::mem::boxed::Box;
use crate::mem::linked_list::{LinkedList, Node};

#[cfg(feature = "log-defmt")]
use defmt::Formatter;
use bern_conf_type::{MemoryLocation, MemoryType};
use crate::mem::queue::PushRaw;
use crate::sync::channel::ChannelError;
use crate::sync::critical_section;
use crate::sync::ipc_channel::{ChannelID, IpcChannelInternal};

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
    /// List of inter-process channels.
    ipc_channels: LinkedList<IpcChannelInternal>,
    /// Channel ID counter, used to assign IDs.
    ipc_channel_counter: AtomicUsize,
}

impl Kernel {
    pub(crate) const fn new() -> Self {
        Kernel {
            state: Cell::new(State::Startup),
            init_process: AtomicPtr::new(null_mut()),
            allocator: UnsafeCell::new(MaybeUninit::uninit()),
            processes: LinkedList::new(),
            ipc_channels: LinkedList::new(),
            ipc_channel_counter: AtomicUsize::new(1),
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
        debug!("Staring kernel.");
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
        critical_section::exec(|| {
            self.processes.push_back(process_node);
        })
    }

    pub(crate) fn is_process_registered(&self, process: &ProcessInternal) -> bool {
        for proc in self.processes.iter() {
            if proc.start_addr() == process.start_addr() {
                return true;
            }
        }
        false
    }

    pub(crate) fn register_channel(&'static self, recv_queue: NonNull<dyn PushRaw>) -> Result<ChannelID, AllocError> {
        let id = self.ipc_channel_counter.load(Ordering::Relaxed);
        self.ipc_channel_counter.fetch_add(1, Ordering::Release);

        critical_section::exec(|| {
            self.ipc_channels.emplace_back(
                IpcChannelInternal::new(id, recv_queue),
                self.allocator()
            ).map(|_| id)
        })
    }

    pub(crate) fn with_channel<F, R>(&self, channel_id: ChannelID, mut f: F) -> Result<R, ChannelError>
        where
            F: FnMut(&mut IpcChannelInternal) -> Result<R, ChannelError>,
    {
        self.ipc_channels
            .iter_mut()
            .find(|ch| ch.id() == channel_id)
            .ok_or(ChannelError::ChannelClosed)
            .and_then(|ch| f(ch))
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

    let mut region_index = 6;
    for memory in CONF.memory_map.additional {
        let (ty, access, exec) = match (memory.memory_type, memory.location) {
            (MemoryType::Rom, _) => (Type::Flash, Access { user: Permission::ReadOnly, system: Permission::ReadOnly }, true),
            (MemoryType::Eeprom, _) => (Type::Flash, Access { user: Permission::ReadWrite, system: Permission::ReadWrite }, true),
            (MemoryType::Flash, _) => (Type::Flash, Access { user: Permission::ReadWrite, system: Permission::ReadWrite }, true),
            (MemoryType::Peripheral, _) => (Type::Peripheral, Access { user: Permission::ReadWrite, system: Permission::ReadWrite }, false),
            (MemoryType::Ram, MemoryLocation::Internal) => (Type::SramInternal, Access { user: Permission::ReadWrite, system: Permission::ReadWrite }, false),
            (MemoryType::Ram, MemoryLocation::External) => (Type::SramExternal, Access { user: Permission::ReadWrite, system: Permission::ReadWrite }, false),
        };

        Arch::enable_memory_region(
            region_index,
            Config {
                    addr: memory.start_address as *const _,
                    memory: ty,
                    size: memory.size,
                    access,
                    executable: exec
            });
        region_index += 1;

        if region_index >= Arch::n_memory_regions() {
            error!("The memory map contains more entries than supported by the memory protection.");
            break;
        }
    }
    for i in region_index..Arch::n_memory_regions() {
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
    info!("Kernel stats");
    info!("============");
    info!("Allocator: {}B/{}B ({}%)",
        KERNEL.allocator().usage().0,
        KERNEL.allocator().capacity().0,
        (KERNEL.allocator().usage().0 as f32 / KERNEL.allocator().capacity().0 as f32 * 100f32) as u8
    );

    info!("Process stats");
    info!("=============");
    info!("Name    Allocator");
    info!("----    ---------");
    for proc in KERNEL.processes.iter() {
        info!("{}", proc);
    }
}