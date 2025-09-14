use crate::mem::boxed::Box;
///
/// Based on [`heapless::llsc`](https://github.com/japaric/heapless/blob/master/src/pool/llsc.rs).
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

type Link<T> = AtomicPtr<Node<T>>;

pub struct Node<T> {
    inner: T,
    next: Link<T>,
}

impl<T> Node<T> {
    pub const fn new(element: T) -> Self {
        Node {
            inner: element,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Queue<T> {
    head: Link<T>,
    len: AtomicUsize,
}

impl<T> Queue<T> {
    pub const fn new() -> Self {
        Queue {
            head: AtomicPtr::new(ptr::null_mut()),
            len: AtomicUsize::new(0),
        }
    }

    pub fn push_front(&self, node: Box<Node<T>>) {
        let node_raw = Box::leak(node);

        loop {
            // CAS loop
            let head = self.head.load(Ordering::Relaxed);
            // Note(unsafe): Pointer requirements are met.
            unsafe { node_raw.as_ref() }
                .next
                .store(head, Ordering::Relaxed);

            match self.head.compare_exchange(
                head,
                node_raw.as_ptr(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.len.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(_) => continue, // loop was interrutped
            }
        }
    }

    pub fn push_back(&self, node: Box<Node<T>>) {
        let node_raw = Box::leak(node);

        loop {
            // CAS loop
            match self.traverse_to_end() {
                None => {
                    match self.head.compare_exchange(
                        ptr::null_mut(),
                        node_raw.as_ptr(),
                        Ordering::Release,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => {
                            self.len.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                        Err(_) => continue, // loop was interrutped
                    }
                }
                Some(back) => {
                    // Note(unsafe): Pointer requirements are met.
                    match unsafe { back.as_ref() }.next.compare_exchange(
                        ptr::null_mut(),
                        node_raw.as_ptr(),
                        Ordering::Release,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => {
                            self.len.fetch_add(1, Ordering::Relaxed);
                            return;
                        }
                        Err(_) => continue, // loop was interrutped
                    }
                }
            }
        }
    }

    pub fn try_pop_front(&self) -> Option<Box<Node<T>>> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            // Note(unsafe): `head` is valid.
            if let Some(node) = unsafe { head.as_mut() } {
                let next = node.next.load(Ordering::Relaxed);
                match self
                    .head
                    .compare_exchange(head, next, Ordering::Release, Ordering::Relaxed)
                {
                    // Note(unsafe): `head` was checked to be non-null.
                    Ok(_) => unsafe {
                        node.next = AtomicPtr::default();
                        self.len.fetch_sub(1, Ordering::Relaxed);
                        return Some(Box::from_raw(NonNull::new_unchecked(node)));
                    },
                    Err(_) => continue,
                }
            } else {
                return None;
            }
        }
    }

    /// Traverses to the end of the list and returns last node if `head` is some.
    fn traverse_to_end(&self) -> Option<NonNull<Node<T>>> {
        let mut node = self.head.load(Ordering::Relaxed);
        if node.is_null() {
            return None;
        }

        // Note(unsafe): Node was checked to be non-null
        unsafe {
            loop {
                let next = (*node).next.load(Ordering::Relaxed);
                if next.is_null() {
                    break;
                }
                node = next;
            }
            Some(NonNull::new_unchecked(node))
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

// Note(unsafe): Queue is lock-free.
unsafe impl<T> Sync for Queue<T> {}
unsafe impl<T> Send for Queue<T> {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    struct MyStruct {
        a: u32,
        b: u8,
    }

    const fn node_array() -> [Node<MyStruct>; 8] {
        [
            Node::new(MyStruct { a: 0, b: 10 }),
            Node::new(MyStruct { a: 1, b: 11 }),
            Node::new(MyStruct { a: 2, b: 12 }),
            Node::new(MyStruct { a: 3, b: 13 }),
            Node::new(MyStruct { a: 4, b: 14 }),
            Node::new(MyStruct { a: 5, b: 15 }),
            Node::new(MyStruct { a: 6, b: 16 }),
            Node::new(MyStruct { a: 7, b: 17 }),
        ]
    }

    #[test]
    fn fifo() {
        static mut LIST_BUFFER: [Node<MyStruct>; 8] = node_array();

        let queue = Queue::new();
        unsafe {
            for n in LIST_BUFFER.iter_mut() {
                queue.push_back(Box::from_raw(NonNull::new_unchecked(n)));
            }
        }
        assert_eq!(queue.len(), 8);

        unsafe {
            let mut i = 0;
            while let Some(node) = queue.try_pop_front() {
                assert_eq!((*node).a, LIST_BUFFER[i].a);
                assert_eq!((*node).b, LIST_BUFFER[i].b);
                Box::leak(node);
                i += 1;
            }
        }
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn lifo() {
        static mut LIST_BUFFER: [Node<MyStruct>; 8] = node_array();

        let queue = Queue::new();
        unsafe {
            for n in LIST_BUFFER.iter_mut() {
                queue.push_front(Box::from_raw(NonNull::new_unchecked(n)));
            }
        }
        assert_eq!(queue.len(), 8);

        unsafe {
            let mut i = LIST_BUFFER.len();
            while let Some(node) = queue.try_pop_front() {
                i -= 1;
                assert_eq!((*node).a, LIST_BUFFER[i].a);
                assert_eq!((*node).b, LIST_BUFFER[i].b);
                Box::leak(node);
            }
        }
        assert_eq!(queue.len(), 0);
    }
}
