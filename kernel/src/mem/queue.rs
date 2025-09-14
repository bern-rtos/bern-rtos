use core::mem::size_of;

pub mod mpmc_linked;
pub mod mpsc_const;
pub mod spsc_const;

#[derive(Debug, PartialEq)]
pub enum QueueError {
    Emtpty,
    Full,
}

/// Basic methods for FIFO Queues.
pub trait FiFoQueue<T, const N: usize> {
    fn try_push_back(&self, item: T) -> Result<(), QueueError>
    where
        T: Copy;
    fn try_pop_front(&self) -> Result<T, QueueError>
    where
        T: Copy;
    fn free(&self) -> usize;
    fn capacity(&self) -> usize;
}

/// If we want to send items via systemcalls and the kernel to different queus,
/// we need to erase the items type.
pub trait PushRaw {
    /// Push a raw item in a queue.
    ///
    /// # Safety
    /// - Item must match the queue type.
    /// - Queue implememtation must assert item size against queue type.
    unsafe fn try_push_back_raw(&self, item: RawItem) -> Result<(), QueueError>;
}

/// Implement this trait if the producer channel is thread safe.
pub unsafe trait SyncProducer {}
/// Implement this trait if the consumer channel is thread safe.
pub unsafe trait SyncConsumer {}

#[derive(Copy, Clone)]
pub struct RawItem {
    ptr: *const usize,
    size: usize,
}

impl RawItem {
    pub fn from<T>(item: &T) -> RawItem {
        RawItem {
            ptr: item as *const _ as *const usize,
            size: size_of::<T>(),
        }
    }
}
