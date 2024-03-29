//! System calls.
//!

use core::alloc::Layout;
use core::mem;
use core::ptr::NonNull;

use crate::{kernel, KERNEL, sched, sync, time};
use crate::sched::event;
use crate::exec::runnable::RunnableResult;
use crate::exec::thread::ThreadBuilder;

use bern_arch::{ICore, ISyscall};
use bern_arch::arch::{Arch, ArchCore};
use bern_arch::core::ExecMode;
use crate::alloc::wrapper::Wrapper;
use crate::alloc::allocator::AllocError;
use crate::mem::queue::{PushRaw, RawItem};
use crate::sync::channel::ChannelError;
use crate::sync::ipc::semaphore::SemaphoreID;


// todo: create with proc macro
// todo: change syscall structure:
//      - arg0: ptr to input
//      - arg1: ptr to result
//      - no arg2 and no return value in r0
// todo: check parameters, how?

#[repr(u8)]
pub enum Service {
    TaskSpawn,
    TaskSleep,
    TaskYield,
    TaskExit,
    EventRegister,
    EventAwait,
    EventFire,
    Alloc,
    Dealloc,
    KernelStats,
    CoreDebugTime,
    TickCount,
    IpcChannelRegister,
    IpcChannelSend,
    IpcSemaphoreRegister,
    IpcSemaphoreTryAcquire,
}

impl Service {
    /// Get syscall service id
    pub const fn service_id(self) -> u8 {
        self as u8
    }
}

fn mode_aware_syscall(service: Service, arg0: usize, arg1: usize, arg2: usize) -> usize {
    match ArchCore::execution_mode() {
        ExecMode::Kernel =>
            syscall_handler_impl(service, arg0, arg1, arg2),
        ExecMode::Thread =>
            Arch::syscall(service.service_id(), arg0, arg1, arg2),
    }
}

/// Add a task to the scheduler based on its `ThreadBuilder` and entry point.
pub(crate) fn thread_spawn(builder: &mut ThreadBuilder, entry: &&mut (dyn FnMut() -> RunnableResult)) {
    mode_aware_syscall(
        Service::TaskSpawn,
        builder as *mut _ as usize,
        entry as *const _ as usize,
        0
    );
}


/// Put the current task to sleep for `ms` milliseconds.
pub fn sleep(ms: u32) {
    mode_aware_syscall(
        Service::TaskSleep,
        ms as usize,
        0,
        0
    );
}

/// Yield the CPU.
///
/// **Note:** If the calling task is the only ready task of its priority it will
/// be put to running state again.
pub fn yield_now() {
    mode_aware_syscall(
        Service::TaskYield,
        0,
        0,
        0
    );
}

/// Terminate the current task voluntarily.
pub fn task_exit() {
    mode_aware_syscall(
        Service::TaskExit,
        0,
        0,
        0
    );
}

/// Allocate and request the ID a new event.
///
/// The ID is later used to await and fire events.
pub(crate) fn event_register() -> usize {
    mode_aware_syscall(
        Service::EventRegister,
        0,
        0,
        0
    )
}

/// Wait until an event or a timeout occurs.
pub(crate) fn event_await(id: usize, timeout: u32) -> Result<(), event::Error> {
    let ret_code = mode_aware_syscall(
        Service::EventAwait,
        id,
        timeout as usize,
        0
    ) as u8;
    unsafe { mem::transmute(ret_code) }
}

/// Trigger an event given its ID.
pub(crate) fn event_fire(id: usize) {
    mode_aware_syscall(
        Service::EventFire,
        id,
        0,
        0
    );
}

pub(crate) fn alloc(layout: Layout) -> *mut u8 {
    mode_aware_syscall(
        Service::Alloc,
        layout.size(),
        layout.align(),
        0,
    ) as *mut u8
}

pub(crate) fn dealloc(ptr: *mut u8, layout: Layout) {
    mode_aware_syscall(
        Service::Dealloc,
        ptr as usize,
        layout.size(),
        layout.align()
    );
}

pub fn print_kernel_stats() {
    mode_aware_syscall(
        Service::KernelStats,
        0,
        0,
        0,
    );
}

pub fn core_debug_time() -> u32 {
    let mut time = 0;
    mode_aware_syscall(
        Service::CoreDebugTime,
        &mut time as *mut _ as usize,
        0,
        0,
    );
    time
}

pub fn tick_count() -> u64 {
    let mut count = 0;
    mode_aware_syscall(
        Service::TickCount,
        &mut count as *mut _ as usize,
        0,
        0,
    );
    count
}

pub(crate) fn ipc_register(recv_channel: &dyn PushRaw) -> Result<usize, AllocError> {
    let mut res = Err(AllocError::Other);
    mode_aware_syscall(
        Service::IpcChannelRegister,
        &recv_channel as *const _ as usize,
        &mut res as *mut _ as usize,
        0
    );
    res
}

pub(crate) fn ipc_send_raw(id: usize, item: RawItem) -> Result<(), ChannelError> {
    let mut res = Err(ChannelError::ChannelClosed);

    mode_aware_syscall(
        Service::IpcChannelSend,
        id,
        &item as *const _ as usize,
        &mut res as *mut _ as usize,
    );
    res
}

pub(crate) fn ipc_semaphore_register() ->  Result<(SemaphoreID, usize), AllocError> {
    let mut res = Err(AllocError::Other);
    mode_aware_syscall(
        Service::IpcSemaphoreRegister,
        0,
        &mut res as *mut _ as usize,
        0
    );
    res
}

pub(crate) fn ipc_semaphore_try_aquire(id: SemaphoreID) -> Result<bool, sync::Error> {
    let mut res = Err(sync::Error::Poisoned);

    mode_aware_syscall(
        Service::IpcSemaphoreTryAcquire,
        &id as *const _ as usize,
        &mut res as *mut _ as usize,
        0
    );
    res
}

// userland barrier ////////////////////////////////////////////////////////////

/// System Call handler.
///
/// **Note:** The syscall above will trigger hardware specific system call
/// handler which **must** call this function.
#[allow(unused_variables)]
fn syscall_handler_impl(service: Service, arg0: usize, arg1: usize, arg2: usize) -> usize {
    #[allow(unreachable_patterns)]
    let r = match service {
        Service::TaskSpawn => {
            let builder: &mut ThreadBuilder = unsafe { mem::transmute(arg0 as *mut ThreadBuilder) };
            let runnable: &&mut (dyn FnMut() -> RunnableResult) = unsafe {
                mem::transmute(arg1 as *mut &mut (dyn FnMut() -> RunnableResult))
            };
            ThreadBuilder::build(
                builder,
                runnable,
            );
            0
        },
        Service::TaskSleep => {
            let ms: u32 = arg0 as u32;
            sched::sleep(ms);
            0
        },
        Service::TaskYield => {
            sched::yield_now();
            0
        },
        Service::TaskExit => {
            sched::task_terminate();
            0
        },

        Service::EventRegister => {
            match sched::event_register() {
                Ok(id) => id,
                Err(_) => 0,
            }
        },
        Service::EventAwait => {
            let id = arg0;
            let timeout = arg1;
            let result = sched::event_await(id, timeout);
            let result: Result<(), event::Error> = Ok(());
            let ret_code: u8 = unsafe { mem::transmute(result) };
            ret_code as usize
        },
        Service::EventFire => {
            let id = arg0;
            sched::event_fire(id);
            0
        },
        Service::Alloc => {
            let size = arg0;
            let align = arg1;
            let layout = unsafe {
                Layout::from_size_align_unchecked(size, align)
            };
            Wrapper::alloc_handler(layout) as usize
        }
        Service::Dealloc => {
            let ptr = arg0 as *mut u8;
            let size = arg1;
            let align = arg2;
            let layout = unsafe {
                Layout::from_size_align_unchecked(size, align)
            };
            Wrapper::dealloc_handler(ptr, layout);
            0
        }
        Service::KernelStats => {
            kernel::print_stats();
            sched::print_thread_stats();
            0
        }
        Service::CoreDebugTime => {
            let time = arg0 as *mut u32;
            unsafe {
                time.write( ArchCore::debug_time())
            }
            0
        }
        Service::TickCount => {
            let count = arg0 as *mut u64;
            unsafe {
                count.write(time::tick_count())
            }
            0
        }
        Service::IpcChannelRegister => {
            let recv_queue: &&dyn PushRaw = unsafe { mem::transmute(arg0) };
            let res: &mut Result<usize, AllocError> = unsafe { mem::transmute(arg1) };
            *res = KERNEL.register_channel(NonNull::from(*recv_queue));
            0
        }
        Service::IpcChannelSend => {
            let id = arg0;
            let item: &RawItem = unsafe { mem::transmute(arg1) };
            let res: &mut Result<(), ChannelError> = unsafe { mem::transmute(arg2) };

            *res = KERNEL.with_channel(id, |ch| {
                ch.push_back(&item)
                    .map_err(ChannelError::Queue)
            });
            0
        }
        Service::IpcSemaphoreRegister => {
            let res: &mut Result<(SemaphoreID, usize), AllocError> = unsafe { mem::transmute(arg1) };
            *res = KERNEL.register_semaphore();
            0
        }
        Service::IpcSemaphoreTryAcquire => {
            let id = &arg0 as &SemaphoreID;
            let res: &mut Result<bool, sync::Error> = unsafe { mem::transmute(arg1) };

            *res = KERNEL.with_semaphore(*id, |s| {
                Ok(false)
            }).or(Err(sync::Error::Poisoned));
            0
        }
        _ => 0,
    };
    r
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "Rust" fn syscall_handler(service: Service, arg0: usize, arg1: usize, arg2: usize) -> usize {
    //trace::isr_enter();
    let r = syscall_handler_impl(service, arg0, arg1, arg2);
    //trace::isr_exit();
    r
}