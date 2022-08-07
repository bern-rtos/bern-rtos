use core::cell::UnsafeCell;

use bern_arch::ISync;
use bern_arch::arch::Arch;
use core::ops::{Deref, DerefMut};

pub struct CriticalMutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> CriticalMutex<T> {
    pub const fn new(element: T) -> Self {
        CriticalMutex {
            inner: UnsafeCell::new(element),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_,T> {
        self.raw_lock();
        MutexGuard::new(&self)
    }

    fn raw_lock(&self) {
        Arch::disable_interrupts(0);
    }

    fn raw_unlock(&self) {
        Arch::enable_interrupts();
    }
}

unsafe impl<T> Sync for CriticalMutex<T> { }

pub struct MutexGuard<'a,T> {
    lock: &'a CriticalMutex<T>,
}

impl<'a,T> MutexGuard<'a,T> {
    fn new(lock: &'a CriticalMutex<T>,) -> Self {
        MutexGuard {
            lock,
        }
    }
}

impl<'a,T> Deref for MutexGuard<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<'a,T> DerefMut for MutexGuard<'a,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}

impl<'a,T> Drop for MutexGuard<'a,T> {
    fn drop(&mut self) {
        self.lock.raw_unlock();
    }
}