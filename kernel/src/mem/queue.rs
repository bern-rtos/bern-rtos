pub mod mpmc_linked;
pub mod mpsc_const;
pub mod spsc_const;


#[derive(Debug, PartialEq)]
pub enum QueueError {
    Emtpty,
    Full,
}

pub trait FiFoQueue<T, const N: usize> {
    fn try_push_back(&self, item: T) -> Result<(), QueueError> where T: Copy;
    fn try_pop_front(&self) -> Result<T, QueueError> where T: Copy;
    fn free(&self) -> usize;
    fn capacity(&self) -> usize;
}

/// Implement this trait if the producer channel is thread safe.
pub unsafe trait SyncProducer { }
/// Implement this trait if the consumer channel is thread safe.
pub unsafe trait SyncConsumer { }
