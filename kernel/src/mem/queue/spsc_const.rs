/// Single Produce Single Consumer queue.
///
/// The queue can send data from *one* thread *one* other thread. This queue has
/// very little memory and runtime overhead.
///
/// Similar to `std::mpmc::channel` and <https://docs.rs/heapless/latest/heapless/spsc/index.html>

use core::cell::UnsafeCell;
use core::mem;
use core::mem::{MaybeUninit, size_of};
use core::sync::atomic::{AtomicU16, Ordering};
use crate::mem::queue::{FiFoQueue, PushRaw, QueueError, RawItem};

pub struct ConstQueue<T, const N: usize> {
    data: [UnsafeCell<MaybeUninit<T>>; N],
    writer: AtomicU16,
    reader: AtomicU16,
}

impl<T, const N: usize> ConstQueue<T, { N }> {
    const INIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    pub const fn new() -> Self {
        ConstQueue {
            data: [Self::INIT; N],
            writer: AtomicU16::new(0),
            reader: AtomicU16::new(0),
        }
    }

    fn free_from_raw(&self, writer: usize, reader: usize) -> usize {
        let capacity = self.capacity();

        if writer >= reader {
            // Writer is between reader and end of data.
            capacity - (writer - reader)
        } else {
            // Writer has wrapped around end of data.
            reader - writer - 1
        }
    }

    fn increment(i: usize) -> usize {
        (i + 1) % N
    }
}

impl<T, const N: usize> FiFoQueue<T, { N }> for ConstQueue<T, { N }> {
    fn try_push_back(&self, item: T) -> Result<(), QueueError> {
        let mut writer= self.writer.load(Ordering::Relaxed) as usize;
        let reader = self.reader.load(Ordering::Relaxed) as usize;

        if self.free_from_raw(writer, reader) == 0 {
            return Err(QueueError::Full);
        }

        writer = Self::increment(writer as usize);

        unsafe {
            (&mut *self.data[writer].get()).write(item);
        }

        self.writer.store(writer as u16, Ordering::Relaxed);
        Ok(())
    }

    fn try_pop_front(&self) -> Result<T, QueueError>
        where T: Copy
    {
        if self.free() == self.capacity() {
            return Err(QueueError::Emtpty);
        }

        let reader = Self::increment(
            self.reader.load(Ordering::Relaxed) as usize
        );

        let item = unsafe {
            (&mut *self.data[reader].get()).assume_init()
        };

        self.reader.store(reader as u16, Ordering::Relaxed);

        Ok(item)
    }

    fn free(&self) -> usize {
        let writer = self.writer.load(Ordering::Relaxed) as usize;
        let reader = self.reader.load(Ordering::Relaxed) as usize;

        self.free_from_raw(writer, reader)
    }

    fn capacity(&self) -> usize {
        N - 1
    }
}

impl<T, const N: usize> PushRaw for ConstQueue<T, { N }>
    where T: Copy
{
    unsafe fn try_push_back_raw(&self, item: RawItem) -> Result<(), QueueError> {
        assert_eq!(item.size, size_of::<T>());
        let item: &mut T = mem::transmute(item.ptr);
        self.try_push_back(*item)
    }
}

unsafe impl<T, const N: usize> Sync for ConstQueue<T, { N }> { }
