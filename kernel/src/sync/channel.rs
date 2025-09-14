use crate::mem::queue::{FiFoQueue, QueueError, SyncConsumer, SyncProducer};
extern crate alloc;
use alloc::boxed::Box;
use core::ptr::NonNull;

#[derive(Debug, PartialEq)]
pub enum ChannelError {
    ChannelClosed,
    Queue(QueueError),
}

pub struct Channel<Q> {
    queue: Q,
}

impl<Q> Channel<Q> {
    pub const fn new(queue: Q) -> Self {
        Channel { queue }
    }

    pub fn split<T, const N: usize>(&'static self) -> (Sender<Q>, Receiver<Q>)
    where
        Q: FiFoQueue<T, { N }>,
        T: Copy,
    {
        unsafe {
            (
                Sender::new(NonNull::new_unchecked(self as *const _ as *mut _)),
                Receiver::new(NonNull::new_unchecked(self as *const _ as *mut _)),
            )
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct Sender<Q> {
    channel: NonNull<Channel<Q>>,
}

impl<Q> Sender<Q> {
    fn new(channel: NonNull<Channel<Q>>) -> Self {
        Sender { channel }
    }

    pub fn send<T, const N: usize>(&self, item: T) -> Result<(), ChannelError>
    where
        Q: FiFoQueue<T, { N }>,
        T: Copy,
    {
        unsafe {
            self.channel
                .as_ref()
                .queue
                .try_push_back(item)
                .map_err(|e| ChannelError::Queue(e))
        }
    }

    pub fn free<T, const N: usize>(&self) -> usize
    where
        Q: FiFoQueue<T, { N }>,
    {
        unsafe { self.channel.as_ref().queue.free() }
    }

    pub fn capacity<T, const N: usize>(&self) -> usize
    where
        Q: FiFoQueue<T, { N }>,
    {
        unsafe { self.channel.as_ref().queue.capacity() }
    }
}

impl<Q> Clone for Sender<Q>
where
    Q: SyncProducer,
{
    fn clone(&self) -> Self {
        Sender {
            channel: self.channel,
        }
    }
}

unsafe impl<Q> Send for Sender<Q> {}

pub struct Receiver<Q> {
    channel: NonNull<Channel<Q>>,
}

impl<Q> Receiver<Q> {
    fn new(channel: NonNull<Channel<Q>>) -> Self {
        Receiver { channel }
    }

    pub fn recv<T, const N: usize>(&self) -> Result<T, ChannelError>
    where
        Q: FiFoQueue<T, { N }>,
        T: Copy,
    {
        unsafe {
            self.channel
                .as_ref()
                .queue
                .try_pop_front()
                .map_err(|e| ChannelError::Queue(e))
        }
    }

    pub fn free<T, const N: usize>(&self) -> usize
    where
        Q: FiFoQueue<T, { N }>,
    {
        unsafe { self.channel.as_ref().queue.free() }
    }

    pub fn capacity<T, const N: usize>(&self) -> usize
    where
        Q: FiFoQueue<T, { N }>,
    {
        unsafe { self.channel.as_ref().queue.capacity() }
    }
}

impl<Q> Clone for Receiver<Q>
where
    Q: SyncConsumer,
{
    fn clone(&self) -> Self {
        Receiver {
            channel: self.channel,
        }
    }
}

unsafe impl<Q> Send for Receiver<Q> {}

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct RefMessage<T> {
    raw: *mut T,
}

impl<T> RefMessage<T> {
    pub fn into_box(self) -> Box<T> {
        unsafe { Box::from_raw(self.raw) }
    }

    pub fn into_mut_ptr(self) -> *mut T {
        self.raw
    }
}

impl<T> From<Box<T>> for RefMessage<T> {
    fn from(message: Box<T>) -> Self {
        RefMessage {
            raw: Box::leak(message),
        }
    }
}

impl<T> From<*mut T> for RefMessage<T> {
    fn from(message: *mut T) -> Self {
        RefMessage { raw: message }
    }
}
