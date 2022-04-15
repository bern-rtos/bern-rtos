use crate::sync::channel::Channel;
use crate::mem::queue::mpsc_const::ConstQueue;

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
        let channel  = channel::<u32, 16>();
        let (tx, rx) = channel.split();

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
    fn queue_overflow() {
        let channel = channel::<u32, 16>();
        let (tx, _rx) = channel.split();

        for i in 0..15 {
            tx.send(i).unwrap();
        }
        assert_eq!(tx.free(), 0);

        let res = tx.send(16);
        assert_eq!(res, Err(ChannelError::Queue(QueueError::Full)));
    }

    #[test]
    fn multi_producer() {
        let channel = channel::<u32, 16>();
        let (tx, rx) = channel.split();

        let tx_a = tx.clone();
        let prod_a = thread::spawn(move || {
            tx_a.send(42).unwrap();
            tx_a.send(43).unwrap();
            tx_a.send(44).unwrap();
            tx_a.send(45).unwrap();
        });
        prod_a.join().unwrap();

        let tx_b = tx.clone();
        let prod_b = thread::spawn(move || {
            tx_b.send(52).unwrap();
            tx_b.send(53).unwrap();
            tx_b.send(54).unwrap();
            tx_b.send(55).unwrap();
        });


        prod_b.join().unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
        assert_eq!(rx.recv().unwrap(), 43);
        assert_eq!(rx.recv().unwrap(), 44);
        assert_eq!(rx.recv().unwrap(), 45);

        assert_eq!(rx.recv().unwrap(), 52);
        assert_eq!(rx.recv().unwrap(), 53);
        assert_eq!(rx.recv().unwrap(), 54);
        assert_eq!(rx.recv().unwrap(), 55);
    }

    #[test]
    fn static_channel() {
        static CHANNEL: Channel<ConstQueue<u32, 16>> = channel::<u32, 16>();
        let (tx, rx) = CHANNEL.split();

        tx.send(42).unwrap();

        assert_eq!(rx.recv().unwrap(), 42);
    }

    #[test]
    fn by_ref() {
        let channel = channel::<RefMessage<u32>, 16>();
        let (tx, rx) = channel.split();

        let a = Box::new(42);
        tx.send(RefMessage::from(a)).unwrap();

        let b = rx.recv().unwrap().into_box();
        assert_eq!(*b, 42);
    }
}