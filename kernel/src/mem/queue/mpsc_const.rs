use crate::mem::queue::{FiFoQueue, QueueError, SyncProducer};
/// Multi Producer Single Consumer queue
///
/// Similar to `std::mpmc::channel`, <https://docs.rs/heapless/latest/heapless/spsc/index.html>
/// and <https://www.codeproject.com/Articles/153898/Yet-another-implementation-of-a-lock-free-circul>
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU16, Ordering};

pub struct ConstQueue<T, const N: usize> {
    data: [UnsafeCell<MaybeUninit<T>>; N],
    /// Position where the next item will be written to.
    writer: AtomicU16,
    /// Uncomplete item. Works as a counting semaphore for the reader.
    staging: AtomicU16,
    /// Reader position.
    reader: AtomicU16,
    /// Last completely written item.
    reader_limit: AtomicU16,
}

impl<T, const N: usize> ConstQueue<T, { N }> {
    const INIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

    pub const fn new() -> Self {
        ConstQueue {
            data: [Self::INIT; N],
            writer: AtomicU16::new(0),
            staging: AtomicU16::new(0),
            reader: AtomicU16::new(0),
            reader_limit: AtomicU16::new(0),
        }
    }

    fn write_acquire(&self) {
        self.staging.fetch_add(1, Ordering::Acquire);
    }

    fn write_release(&self) {
        // If there are no staging write operations we can move the read limit
        // to the writer position.
        let reader_limit = self.reader_limit.load(Ordering::Relaxed);
        let writer = self.writer.load(Ordering::Relaxed);
        self.staging.fetch_sub(1, Ordering::Release);

        if self.staging.load(Ordering::Relaxed) == 0 {
            // The exchange operation will fail if this function is interrupted
            // and `self.reader_limit` was advanced from the other
            // thread/interrupt. In this case the `reader_limit` is already
            // correct and we do not need to store it again.
            self.reader_limit
                .compare_exchange(reader_limit, writer, Ordering::Relaxed, Ordering::Relaxed)
                .ok();
            self.reader_limit.store(writer, Ordering::Relaxed);
        }
    }

    fn free_from_raw(&self, writer: usize, reader: usize) -> usize {
        let capacity = self.capacity();

        if writer >= reader {
            // Writer is between reader and end of data.
            capacity - (writer - reader)
        } else {
            // Writer has wrapped around end of data.
            capacity - reader - writer
        }
    }

    fn increment(i: usize) -> usize {
        (i + 1) % N
    }
}

impl<T, const N: usize> FiFoQueue<T, { N }> for ConstQueue<T, { N }> {
    fn try_push_back(&self, item: T) -> Result<(), QueueError> {
        let reader = self.reader.load(Ordering::Relaxed) as usize;
        let mut writer;

        self.write_acquire();

        // Try to take an item
        loop {
            // CAS loop
            writer = self.writer.load(Ordering::Relaxed) as usize;

            if Self::increment(writer) == reader {
                self.write_release();
                return Err(QueueError::Full);
            }

            match self.writer.compare_exchange(
                writer as u16,
                Self::increment(writer) as u16,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => {}
            }
        }

        unsafe {
            (&mut *self.data[Self::increment(writer)].get()).write(item);
        }

        self.write_release();

        Ok(())
    }

    fn try_pop_front(&self) -> Result<T, QueueError>
    where
        T: Copy,
    {
        let mut reader = self.reader.load(Ordering::Relaxed) as usize;
        let limit = self.reader_limit.load(Ordering::Relaxed) as usize;

        if reader == limit {
            return Err(QueueError::Emtpty);
        }

        reader = Self::increment(reader);

        let item = unsafe { (&mut *self.data[reader].get()).assume_init() };

        self.reader.store(reader as u16, Ordering::Relaxed);

        Ok(item)
    }

    fn free(&self) -> usize {
        let limit = self.reader_limit.load(Ordering::Relaxed) as usize;
        let reader = self.reader.load(Ordering::Relaxed) as usize;

        self.free_from_raw(limit, reader)
    }

    fn capacity(&self) -> usize {
        N - 1
    }
}

unsafe impl<T, const N: usize> Sync for ConstQueue<T, { N }> {}

unsafe impl<T, const N: usize> SyncProducer for ConstQueue<T, { N }> {}
