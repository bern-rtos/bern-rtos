use crate::mem::queue::spsc_const::ConstQueue;
use crate::sync::channel::Channel;

pub const fn channel<T, const N: usize>() -> Channel<ConstQueue<T, { N }>> {
    Channel::new(ConstQueue::new())
}


#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use std::thread;
    use std::boxed::Box;
    use crate::mem::queue::QueueError;
    use crate::sync::channel::{ChannelError, RefMessage};

    #[test]
    fn single_producer() {
        static CHANNEL: Channel<ConstQueue<u32, 16>>  = channel::<u32, 16>();
        let (tx, rx) = CHANNEL.split();

        assert_eq!(rx.free(), 15);
        tx.send(42).unwrap();
        assert_eq!(rx.free(), 14);
        tx.send(43).unwrap();
        assert_eq!(rx.free(), 13);
        tx.send(44).unwrap();
        assert_eq!(rx.free(), 12);
        tx.send(45).unwrap();
        assert_eq!(rx.free(), 11);

        assert_eq!(rx.recv().unwrap(), 42);
        assert_eq!(rx.recv().unwrap(), 43);
        assert_eq!(rx.recv().unwrap(), 44);
        assert_eq!(rx.recv().unwrap(), 45);
        assert_eq!(rx.free(), 15);
    }

    #[test]
    fn overflow() {
        static CHANNEL: Channel<ConstQueue<u32, 16>>  = channel::<u32, 16>();
        let (tx, _rx) = CHANNEL.split();

        for i in 0..15 {
            tx.send(i).unwrap();
        }
        assert_eq!(tx.free(), 0);

        let res = tx.send(16);
        assert_eq!(res, Err(ChannelError::Queue(QueueError::Full)));
    }

    #[test]
    fn underflow() {
        static CHANNEL: Channel<ConstQueue<u32, 16>>  = channel::<u32, 16>();
        let (tx, rx) = CHANNEL.split();

        for i in 0..5 {
            tx.send(i).unwrap();
        }
        assert_eq!(tx.free(), 10);

        for _i in 0..5 {
            rx.recv().unwrap();
        }
        assert_eq!(rx.free(), 15);

        let res = rx.recv();
        assert_eq!(res, Err(ChannelError::Queue(QueueError::Emtpty)));
    }

    #[test]
    fn spsc_thread() {
        static CHANNEL: Channel<ConstQueue<u32, 16>>  = channel::<u32, 16>();
        let (tx, rx) = CHANNEL.split();

        let prod_a = thread::spawn(move || {
            tx.send(42).unwrap();
            tx.send(43).unwrap();
            tx.send(44).unwrap();
            tx.send(45).unwrap();
        });

        prod_a.join().unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
        assert_eq!(rx.recv().unwrap(), 43);
        assert_eq!(rx.recv().unwrap(), 44);
        assert_eq!(rx.recv().unwrap(), 45);
    }

    #[test]
    fn static_channel() {
        static CHANNEL: Channel<ConstQueue<u32, 16>>  = channel::<u32, 16>();
        let (tx, rx) = CHANNEL.split();
        tx.send(42).unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn by_ref() {
        static CHANNEL: Channel<ConstQueue<RefMessage<u32>, 16>>  = channel::<RefMessage<u32>, 16>();
        let (tx, rx) = CHANNEL.split();

        let a = Box::new(42);
        tx.send(RefMessage::from(a)).unwrap();

        let b = rx.recv().unwrap().into_box();
        assert_eq!(*b, 42);
    }
}