use core::marker::PhantomData;
use core::ptr::NonNull;
use crate::mem::queue::{FiFoQueue, PushRaw, QueueError, RawItem};
use crate::sync::channel::ChannelError;
use crate::syscall;

pub(crate) type ChannelID = usize;

pub struct IpcChannel;

impl IpcChannel{
    pub fn new() -> Self {
        IpcChannel
    }

    pub fn split<Q, T, const N: usize>(self, receive_queue: &'static Q) -> Result<(IpcSender<T>, IpcReceiver<Q>), ChannelError>
        where
            Q: FiFoQueue<T, { N }> + PushRaw,
            T: Copy,
    {
        let id = syscall::ipc_register(receive_queue)
            .map_err(|_e| ChannelError::ChannelClosed)?;

        Ok((IpcSender::<T>::new(id), IpcReceiver::new(id, receive_queue)))
    }
}


///////////////////////////////////////////////////////////////////////////////

pub struct IpcSender<T> {
    channel_id: ChannelID,
    _marker: PhantomData<T>
}

impl<T> IpcSender<T>
    where T: Copy
{
    fn new(channel_id: ChannelID) -> Self {
        IpcSender {
            channel_id,
            _marker: PhantomData::default(),
        }
    }

    pub fn send(&self, item: T) -> Result<(), ChannelError> {
        let raw = RawItem::from(&item);
        syscall::ipc_send_raw(self.channel_id, raw)
    }
}

unsafe impl<T> Send for IpcSender<T> { }


///////////////////////////////////////////////////////////////////////////////

pub struct IpcReceiver<Q: 'static> {
    _channel_id: ChannelID,
    recv_queue: &'static Q,
}

impl<Q: 'static> IpcReceiver<Q> {
    fn new(channel_id: ChannelID, recv_queue: &'static Q) -> Self {
        IpcReceiver {
            _channel_id: channel_id,
            recv_queue
        }
    }

    pub fn recv<T, const N: usize>(&self) -> Result<T, ChannelError>
        where
            Q: FiFoQueue<T, { N }>,
            T: Copy,
    {
        // todo: wait for kernel event

        self.recv_queue.try_pop_front()
            .map_err(|e| ChannelError::Queue(e))
    }

    pub fn free<T, const N: usize>(&self) -> usize
        where Q: FiFoQueue<T, { N }>
    {
        self.recv_queue.free()
    }

    pub fn capacity<T, const N: usize>(&self) -> usize
        where Q: FiFoQueue<T, { N }>
    {
        self.recv_queue.capacity()
    }
}

unsafe impl<Q> Send for IpcReceiver<Q> { }

///////////////////////////////////////////////////////////////////////////////


pub(crate) struct IpcChannelInternal {
    channel_id: ChannelID,
    recv_queue: NonNull<dyn PushRaw>,
}

impl IpcChannelInternal {
    pub fn new(channel_id: ChannelID, recv_queue: NonNull<dyn PushRaw>) -> IpcChannelInternal {
        IpcChannelInternal {
            channel_id,
            recv_queue,
        }
    }

    pub fn id(&self) -> ChannelID {
        self.channel_id
    }

    pub fn push_back(&self, item: &RawItem) -> Result<(), QueueError> {
        unsafe { self.recv_queue.as_ref().try_push_back_raw(*item) }
    }
}