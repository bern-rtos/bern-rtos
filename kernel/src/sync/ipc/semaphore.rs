use crate::alloc::allocator::AllocError;
use crate::sched::event;
use crate::sync::Error;
use crate::syscall;

pub type SemaphoreID = usize;

pub struct IpcSemaphore {
    semaphore_id: SemaphoreID,
    event_id: usize,
}

impl IpcSemaphore {
    pub fn new() -> Result<IpcSemaphore, AllocError> {
        let (s, e) = syscall::ipc_semaphore_register()?;
        Ok(IpcSemaphore {
            semaphore_id: s,
            event_id: e,
        })
    }

    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, Error> {
        syscall::ipc_semaphore_try_aquire(self.semaphore_id).and_then(|v| {
            if v {
                Ok(SemaphorePermit::new(self))
            } else {
                Err(Error::WouldBlock)
            }
        })
    }

    pub fn acquire(&self, timeout: u32) -> Result<SemaphorePermit<'_>, Error> {
        syscall::ipc_semaphore_try_aquire(self.semaphore_id).and_then(|v| {
            if v {
                Ok(SemaphorePermit::new(self))
            } else {
                match syscall::event_await(self.event_id, timeout) {
                    Ok(_) => {
                        syscall::ipc_semaphore_try_aquire(self.semaphore_id).ok();
                        Ok(SemaphorePermit::new(self))
                    }
                    Err(event::Error::TimeOut) => Err(Error::TimeOut),
                    Err(_) => Err(Error::Poisoned),
                }
            }
        })
    }

    pub fn available_permits(&self) -> usize {
        0
    }

    pub fn add_permits(&self, _n: usize) {}
}

unsafe impl Sync for IpcSemaphore {}

// todo: add traits for semaphore and make permit generic
/// Scoped semaphore permit
///
/// similar to [`tokio::sync::SemaphorePermit`](https://docs.rs/tokio/0.2.6/tokio/sync/struct.SemaphorePermit.html).
pub struct SemaphorePermit<'a> {
    _semaphore: &'a IpcSemaphore,
}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a IpcSemaphore) -> Self {
        SemaphorePermit {
            _semaphore: semaphore,
        }
    }

    /// Forget permit. Will not be returned to the available permits.
    pub fn forget(self) {}
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {}
}

pub(crate) struct IpcSemaphoreInternal {
    semaphore_id: SemaphoreID,
    _event_id: usize,
}

impl IpcSemaphoreInternal {
    pub(crate) fn new(semaphore_id: SemaphoreID, event_id: usize) -> IpcSemaphoreInternal {
        IpcSemaphoreInternal {
            semaphore_id,
            _event_id: event_id,
        }
    }

    pub(crate) fn id(&self) -> SemaphoreID {
        self.semaphore_id
    }
}
