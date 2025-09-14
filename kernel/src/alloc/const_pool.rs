use crate::log;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{mem, ptr};

pub struct Item<T> {
    data: MaybeUninit<T>,
    lock: AtomicBool,
}

impl<T> Item<T> {
    const fn new() -> Self {
        Item {
            data: MaybeUninit::uninit(),
            lock: AtomicBool::new(false),
        }
    }

    pub fn locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    pub fn try_acquire(&self) -> Option<ConstBox<T>> {
        match self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            // Note(unsafe): self is guaranteed to be non-null
            Ok(_) => Some(ConstBox::new(unsafe {
                NonNull::new_unchecked(self as *const _ as *mut _)
            })),
            Err(_) => None,
        }
    }
}

impl<T> Deref for Item<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.data.as_ptr()) }
    }
}

impl<T> DerefMut for Item<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.data.as_mut_ptr()) }
    }
}

impl<T> Drop for Item<T> {
    fn drop(&mut self) {
        log::trace!("Dropping const item.");
        self.lock.store(false, Ordering::Release);
    }
}

pub struct ConstPool<T, const N: usize> {
    items: [Item<T>; N],
}

impl<T, const N: usize> ConstPool<T, { N }> {
    const INIT: Item<T> = Item::new();

    pub const fn new() -> Self {
        ConstPool {
            items: [Self::INIT; N],
        }
    }

    pub fn try_acquire(&self) -> Option<ConstBox<T>> {
        for i in self.items.iter() {
            if !i.locked() {
                match i.try_acquire() {
                    None => continue,
                    Some(b) => return Some(b),
                }
            }
        }
        None
    }

    pub fn free(&self) -> usize {
        let mut free = 0;
        for i in self.items.iter() {
            if !i.locked() {
                free += 1;
            }
        }
        free
    }
}

unsafe impl<T, const N: usize> Sync for ConstPool<T, { N }> {}

pub struct ConstBox<T> {
    item: NonNull<Item<T>>,
}

impl<T> ConstBox<T> {
    const fn new(item: NonNull<Item<T>>) -> Self {
        ConstBox { item }
    }

    pub fn leak(b: Self) -> NonNull<Item<T>> {
        let boxed = b.item;
        mem::forget(b);
        boxed
    }

    pub unsafe fn from_raw(item: NonNull<Item<T>>) -> Self {
        ConstBox { item }
    }
}

impl<T> Deref for ConstBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.item.as_ptr()).deref() }
    }
}

impl<T> DerefMut for ConstBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { (*self.item.as_ptr()).deref_mut() }
    }
}

impl<'a, T> Drop for ConstBox<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.item.as_ptr());
        }
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    struct SomeData {
        pub a: u32,
        pub b: u32,
    }

    #[test]
    fn drop_item() {
        let pool = ConstPool::<SomeData, 10>::new();

        {
            let mut item = pool.try_acquire().unwrap();
            (*item).a = 10;
            (*item).b = 100;
            assert_eq!(pool.free(), 9);
        } // Drop element
        assert_eq!(pool.free(), 10);
    }
}
