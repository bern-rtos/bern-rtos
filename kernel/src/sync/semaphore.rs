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
/// # Example
/// ```no_run
/// // put the mutex in a section where all tasks have access
/// #[link_section = ".shared"]
/// // create a counting semaphore that can be taken 4 times
/// static SEMAPHORE: Semaphore = Semaphore::new(4);
///
/// fn main() -> ! {
///     /*...*/
///     sched::init();
///     // allocate an event slot in the kernel, so that tasks can wait for a permit
///     SEMAPHORE.register().ok();
///
///     Task::new()
///         .static_stack(kernel::alloc_static_stack!(512))
///         .spawn(move || {
///             loop {
///                 /*...*/
///                 // attempt to acquire a permit with timeout
///                 match SEMAPHORE.acquire(1000) {
///                     do_something();
///                 }; // permit released automatically
///            }
///         });
/// }
/// ```
pub struct Semaphore {
    id: UnsafeCell<usize>,
    permits: AtomicUsize,
    permits_issued: AtomicUsize,
}

impl Semaphore {
    pub const fn new(permits: usize) -> Self {
        Semaphore {
            id: UnsafeCell::new(0),
            permits:  AtomicUsize::new(permits),
            permits_issued: AtomicUsize::new(0),
        }
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
            unsafe { self.id.get().write(id); }
            Ok(())
        }
    }

    /// Try to acquire a semaphore permit (non-blocking). Returns a
    /// [`SemaphorePermit`] or an error if no permit is available or semaphore
    /// is poisoned.
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, Error> {
        if self.raw_acquire().is_ok() {
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
        if self.raw_acquire().is_ok() {
            return Ok(SemaphorePermit::new(&self));
        } else {
            let id = unsafe { *self.id.get() };
            match syscall::event_await(id, timeout) {
                Ok(_) => {
                    self.raw_acquire().ok();
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
        syscall::event_fire(unsafe { *self.id.get() });
    }

    /// **Note:** This will return a false positive when `permits_issued` overflows
    fn raw_acquire(&self) -> Result<(), ()> {
        let permits = self.permits_issued.fetch_add(1, Ordering::Acquire);
        if permits >= self.permits.load(Ordering::Relaxed) {
            self.permits_issued.fetch_sub(1, Ordering::Release);
            Err(())
        } else {
            Ok(())
        }

    }

    fn raw_release(&self) {
        self.permits_issued.fetch_sub(1, Ordering::Release);
        // NOTE(unsafe): `id` is not changed after startup
        syscall::event_fire(unsafe { *self.id.get() });
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