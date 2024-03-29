use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use super::Error;
use crate::syscall;
use crate::sched::event;

/// Atomic counting semaphore
///
/// similar to [`Struct tokio::sync::Semaphore`](https://docs.rs/tokio/0.2.6/tokio/sync/struct.Semaphore.html)
/// and [`super::Mutex`].
///
/// A semaphore can be used to
/// - synchronize on one or more event (e.g. interrupt)
/// - synchronize multiple tasks
///
pub struct Semaphore {
    event_id: UnsafeCell<usize>,
    permits: AtomicUsize,
    permits_issued: AtomicUsize,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        let semaphore = Semaphore {
            event_id: UnsafeCell::new(0),
            permits:  AtomicUsize::new(permits),
            permits_issued: AtomicUsize::new(0),
        };

        semaphore.register().ok();
        semaphore
    }

    /// Allocate an event ot the semaphore.
    ///
    /// **Note:** The kernel must be initialized before calling this method.
    pub fn register(&self) -> Result<(),Error> {
        let id = syscall::event_register();
        if id == 0 {
            Err(Error::OutOfMemory)
        } else {
            // NOTE(unsafe): only called before the semaphore is in use
            unsafe { self.event_id.get().write(id); }
            Ok(())
        }
    }

    /// Try to acquire a semaphore permit (non-blocking). Returns a
    /// [`SemaphorePermit`] or an error if no permit is available or semaphore
    /// is poisoned.
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, Error> {
        if self.raw_try_acquire() {
            Ok(SemaphorePermit::new(&self))
        } else {
            Err(Error::WouldBlock)
        }
    }

    /// Try to acquire a semaphore permit (blocking). Returns a
    /// [`SemaphorePermit`] or an error if the request timed out or the semaphore
    /// was poisoned.
    ///
    /// **Note:** The timeout function is not implemented yet.
    pub fn acquire(&self, timeout: u32) ->  Result<SemaphorePermit<'_>, Error> {
        if self.raw_try_acquire() {
            return Ok(SemaphorePermit::new(&self));
        } else {
            let id = unsafe { *self.event_id.get() };
            match syscall::event_await(id, timeout) {
                Ok(_) => {
                    self.raw_try_acquire();
                    Ok(SemaphorePermit::new(&self))
                },
                Err(event::Error::TimeOut) => Err(Error::TimeOut),
                Err(_) => Err(Error::Poisoned),
            }
        }
    }

    /// Number of permits that can be issued from this semaphore
    pub fn available_permits(&self) -> usize {
        self.permits.load(Ordering::Relaxed) - self.permits_issued.load(Ordering::Relaxed)
    }

    // Add `n` new permits to the semaphore.
    pub fn add_permits(&self, n: usize) {
        self.permits.fetch_add(n, Ordering::Release);
        // NOTE(unsafe): `id` is not changed after startup
        syscall::event_fire(unsafe { *self.event_id.get() });
    }

    fn raw_try_acquire(&self) -> bool {
        loop { // CAS loop
            let permits_issued = self.permits_issued.load(Ordering::Acquire);
            if permits_issued < self.permits.load(Ordering::Relaxed) {
                match self.permits_issued.compare_exchange(
                    permits_issued,
                    permits_issued + 1,
                    Ordering::Release,
                    Ordering::Relaxed
                ) {
                    Ok(_) =>  { return true; }
                    Err(_) => continue, // CAS loop was interrupted, restart
                }
            } else {
                return false;
            }
        }
    }

    fn raw_release(&self) {
        self.permits_issued.fetch_sub(1, Ordering::Release);
        // NOTE(unsafe): `id` is not changed after startup
        syscall::event_fire(unsafe { *self.event_id.get() });
    }
}

unsafe impl Sync for Semaphore {}


/// Scoped semaphore permit
///
/// similar to [`tokio::sync::SemaphorePermit`](https://docs.rs/tokio/0.2.6/tokio/sync/struct.SemaphorePermit.html).
pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        SemaphorePermit {
            semaphore,
        }
    }

    /// Forget permit. Will not be returned to the available permits.
    pub fn forget(self) {
        self.semaphore.permits.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        self.semaphore.raw_release();
    }
}