//! Doubly-linked list.
//!
//! The goal here is to create a fast and efficient linked list.
//! Lists use an array of nodes as memory pool, the array must be static.
//!
//! In contrast to [`std::collections::LinkedList`](https://doc.rust-lang.org/alloc/collections/linked_list/struct.LinkedList.html)
//! you will only ever get a reference to a node and never a copy/move.

#![allow(unused)]

use core::ptr;
use core::ptr::NonNull;
use core::mem::MaybeUninit;
use core::cell::RefCell;
use core::borrow::BorrowMut;
use crate::mem::boxed::Box;
use crate::mem::pool_allocator::{self, PoolAllocator};
use core::ops::{Deref, DerefMut};

type Link<T> = Option<NonNull<Node<T>>>;

/******************************************************************************/

/// An element/node of a list.
// Copy needed for initialization
#[derive(Debug, Copy, Clone)]
pub struct Node<T> {
    inner: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Node<T> {
    /// Create a node from an element
    pub fn new(element: T) -> Self {
        Node {
            inner: element,
            prev: None,
            next: None,
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

/******************************************************************************/

/// A doubly-linked list owning its nodes.
///
/// Based on [std::collections::LinkedList](https://doc.rust-lang.org/alloc/collections/linked_list/struct.LinkedList.html)
/// and <https://rust-unofficial.github.io/too-many-lists>.
///
/// # Examples
///
/// Create a new list with `ArrayPool` allocator:
/// ```no_run
/// static POOL: ArrayPool<Node<MyStruct>,16> = ArrayPool::new([None; 16]);
/// let mut list_a = LinkedList::new(&POOL);
/// let mut list_b = LinkedList::new(&POOL);
/// ```
///
/// Add element to the end of a list:
/// ```
/// list_a.emplace_back(MyStruct { id: 42 });
/// list_a.emplace_back(MyStruct { id: 54 });
///```
///
/// Move an element from one to another list:
/// ```
/// let node = list_a.pop_front();
/// list_a.push_back(node);
///```
#[derive(Debug)]
pub struct LinkedList<T,P>
    where P: PoolAllocator<Node<T>> + 'static
{
    head: Link<T>,
    tail: Link<T>,
    pool: &'static P,
    len: usize,
}

impl<T,P> LinkedList<T,P>
    where P: PoolAllocator<Node<T>> + 'static
{
    /// Create a new list from an allocator
    pub fn new(pool: &'static P) -> Self {
        LinkedList {
            head: None,
            tail: None,
            pool,
            len: 0,
        }
    }

    /// Allocate a new element and move it to the end of the list
    ///
    /// **Note:** This fails when we're out of memory
    pub fn emplace_back(&mut self, element: T) -> Result<(), pool_allocator::Error> {
        let node = self.pool.insert(Node::new(element));
        node.map(|n| {
            self.push_back(n);
        })
    }

    /// Insert a node at the end on the list
    pub fn push_back(&mut self, mut node: Box<Node<T>>) {
        node.prev = self.tail;

        let link = Some(node.into_nonnull());
        // NOTE(unsafe):  we check tail is Some()
        unsafe {
            match self.tail {
                None => self.head = link,
                Some(mut tail) => tail.as_mut().next = link,
            }
        }

        self.tail = link;
        self.len += 1;
    }

    /// Remove and return the first node from the list if there is any
    pub fn pop_front(&mut self) -> Option<Box<Node<T>>> {
        let mut front = self.head.take();

        match front {
            Some(mut node) => unsafe {
                self.head = node.as_ref().next;
                if let Some(mut head) = self.head {
                    head.as_mut().prev = None;
                }
                if self.tail == Some(node) {
                    self.tail = node.as_ref().next;
                }
                node.as_mut().next = None;
                self.len -= 1;
                Some(Box::from_raw(node))
            },
            None => None,
        }
    }

    /// Insert a node exactly before a given node
    ///
    /// **Note:** prefer [`Self::insert_when()`] if possible
    pub fn insert(&mut self, mut node: Box<Node<T>>, mut new_node: Box<Node<T>>) {
        let mut node = node.into_nonnull();
        let mut new_node = new_node.into_nonnull();
        unsafe {
            match node.as_mut().prev {
                Some(mut prev) => prev.as_mut().next = Some(new_node),
                None => self.head = Some(new_node),
            }
            node.as_mut().prev = Some(new_node);
            new_node.as_mut().next = Some(node);
        }
        self.len += 1;
    }

    /// Insert a node before the first failed match given a comparison criteria
    ///
    /// # Example
    /// Insert task `pausing` before the element where the next wake-up time
    /// `next_wut()` is larger than the one of `pausing`.
    /// ```no_run
    /// /* create and populate list */
    /// let pausing: Task = /* omitted */;
    /// tasks_sleeping.insert_when(
    ///     pausing,
    ///     |pausing, task| {
    ///         pausing.next_wut() < task.next_wut()
    ///     });
    /// ```
    pub fn insert_when(&mut self, mut node: Box<Node<T>>, criteria: impl Fn(&T, &T) -> bool) {
        if let Some(mut current) = self.head {
            loop { unsafe {
                if criteria(&*node, &*current.as_ref()) {
                    self.insert(Box::from_raw(current), node);
                    return;
                }
                current = match current.as_ref().next {
                    Some(node) => node,
                    None => break,
                }
            }}
        }
        self.push_back(node);
    }

    /// Get a reference to the first value of the list if there is a node
    pub fn front(&self) -> Option<&T> {
        self.head.map(|front| unsafe { &**front.as_ref() })
    }

    /// Get a reference to last value of the list if there is a node
    pub fn back(&self) -> Option<&T> {
        self.tail.map(|back| unsafe { &**back.as_ref() })
    }

    /// Get the current length of the list
    pub fn len(&self) -> usize {
        self.len
    }

    /// Remove a node from any point in the list.
    ///
    /// # Safety
    /// A node is only allowed to be unliked once.
    unsafe fn unlink(&mut self, node: &mut Node<T>) -> Box<Node<T>> {
        match node.prev {
            Some(mut prev) => prev.as_mut().next = node.next,
            None => self.head = node.next,
        };

        match node.next {
            Some(mut next) => next.as_mut().prev = node.prev,
            None => self.tail = node.prev,
        };

        node.prev = None;
        node.next = None;
        self.len -= 1;

        Box::from_raw(NonNull::new_unchecked(node))
    }

    /// Provides a forward iterator.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.map(|node| unsafe { & *node.as_ptr() }),
        }
    }

    /// Provides a forward iterator with mutable references.
    pub fn iter_mut(&self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.map(|node| unsafe { &mut *node.as_ptr() })
        }
    }

    /// Provides a cursor with editing operation at the front element.
    pub fn cursor_front_mut(&mut self) -> Cursor<'_, T, P> {
        Cursor { node: self.head, list: self }
    }
}

/******************************************************************************/

/// An iterator over the elements of a [`LinkedList`].
///
/// This `struct` is created by [`LinkedList::iter()`].
pub struct Iter<'a, T>
{
    next: Option<&'a Node<T>>,
}

impl<'a,T> Iterator for Iter<'a,T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| unsafe {
            self.next = node.next.map(|next| next.as_ref());
            &**node
        })
    }
}

/// An mutable iterator over the elements of a [`LinkedList`].
///
/// This `struct` is created by [`LinkedList::iter_mut()`].
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| unsafe {
            self.next = node.next.map(|mut next| next.as_mut());
            &mut **node
        })
    }
}

/******************************************************************************/

/// A cursor over a [`LinkedList`] with editing operations.
///
/// In contrast to an iterator a cursor can move from front to back and take an
/// element out of the list.
#[derive(Debug)]
pub struct Cursor<'a,T,P>
    where P: PoolAllocator<Node<T>> + 'static
{
    node: Link<T>,
    list: &'a mut LinkedList<T,P>,
}

impl<'a, T, P> Cursor<'a, T, P>
    where P: PoolAllocator<Node<T>> + Sized
{
    /// Get reference to value of node if there is any
    pub fn inner(&self) -> Option<&T> {
        self.node.map(|node| unsafe { &**node.as_ref() })
    }

    /// Get mutable reference to value of node if there is any
    pub fn inner_mut(&self) -> Option<&mut T> {
        self.node.map(|mut node| unsafe { &mut **node.as_mut() })
    }

    /// Move cursor to the next node
    pub fn move_next(&mut self) {
        if let Some(node) = self.node {
            self.node = unsafe { node.as_ref().next };
        }
    }

    /// Take the current node if there is one
    pub fn take(&mut self) -> Option<Box<Node<T>>> {
        self.node.map(|mut node|
            unsafe {
                self.node = node.as_ref().next;
                self.list.unlink(node.as_mut())
            })
    }
}

/******************************************************************************/

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use core::borrow::Borrow;
    use crate::mem::array_pool::ArrayPool;

    type Pool = ArrayPool<Node<MyStruct>,16>;

    #[derive(Debug, Copy, Clone)]
    struct MyStruct {
        pub id: u32,
    }

    #[test]
    fn one_node() {
        static POOL: Pool = ArrayPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        assert_eq!(node_0.prev, None);
        assert_eq!(node_0.next, None);

        let mut list = LinkedList::new(&POOL);
        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);

        list.push_back(node_0);
        assert_ne!(list.head, None);
        assert_eq!(list.tail, list.head);
        unsafe {
            assert_eq!(list.head.unwrap().as_ref().prev, None);
            assert_eq!(list.head.unwrap().as_ref().next, None);
        }

        let node = list.pop_front();

        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);
        assert_eq!(node.as_ref().unwrap().prev, None);
        assert_eq!(node.as_ref().unwrap().next, None);
    }

    #[test]
    fn length() {
        static POOL: Pool = ArrayPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        assert_eq!(list.len(), 0);
        list.pop_front();
        assert_eq!(list.len(), 0);
        list.emplace_back(MyStruct { id: 42 });
        assert_eq!(list.len(), 1);
        list.pop_front();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn pushing_and_popping() {
        static POOL: Pool = ArrayPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        list.emplace_back(MyStruct { id: 42 });
        list.emplace_back(MyStruct { id: 43 });

        let mut another_list = LinkedList::new(&POOL);
        list.emplace_back(MyStruct { id: 44 });

        let mut front = list.pop_front();
        assert_eq!(front.as_mut().unwrap().inner().id, 42);
        another_list.push_back(front.unwrap());

        assert_eq!(another_list.back().unwrap().id, 42);
    }

    #[test]
    fn pool_overflow() {
        static POOL: Pool = ArrayPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        for i in 0..16 {
            assert_eq!(list.emplace_back(MyStruct { id: i }), Ok(()));
        }
        assert_eq!(list.emplace_back(MyStruct { id: 16 }), Err(pool_allocator::Error::OutOfMemory));
    }

    #[test]
    fn iterate() {
        static POOL: Pool = ArrayPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let truth = vec![42,43,44,45];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
        // everything should still work fine
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
    }

    #[test]
    fn iterate_mut() {
        static POOL: Pool = ArrayPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let truth = vec![42,43,44,45];
        for (i, element) in list.iter_mut().enumerate() {
            assert_eq!(element.id, truth[i]);
            element.id = i as u32;
        }
        // values should have changed
        let truth = vec![0,1,2,3];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
    }

    #[test]
    fn find_and_take() {
        static POOL: Pool = ArrayPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let mut another_list = LinkedList::new(&POOL);

        let mut cursor = list.cursor_front_mut();
        let mut target: Option<Box<Node<MyStruct>>> = None;
        while let Some(element) = cursor.inner() {
            if element.id == 43 {
                target = cursor.take();
                break;
            }
            cursor.move_next();
        }
        another_list.push_back(target.unwrap());

        let truth = vec![42,44];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }

        for element in another_list.iter() {
            assert_eq!(element.id, 43);
        }
    }
}