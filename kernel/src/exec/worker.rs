use core::cell::Cell;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};
use core::ops::{Deref, DerefMut};
use crate::alloc::allocator::AllocError;
use crate::alloc::const_pool::{ConstBox, Item};
use crate::exec::process;
use crate::exec::process::ProcessInternal;
use crate::exec::runnable::Priority;
use crate::exec::thread::Thread;
use crate::mem::boxed::Box;
use crate::mem::queue::mpmc_linked::{Node, Queue};
use crate::stack::Stack;
use crate::{log, syscall};

//pub trait WorkTrait: 'static + FnOnce() { }
pub trait Workable {
    fn process(&self);
    fn release(&self);
}

pub struct WorkItem<T> {
    owner: NonNull<Item<WorkItem<T>>>,
    trait_node: Node<&'static dyn Workable>,
    data: T,
    function: fn(&T),
}

impl<T> WorkItem<T> {

    fn trait_node(&'static mut self) -> &Node<&dyn Workable> {
        // todo: remove this lifetime hack
        let self_ref = unsafe { &mut *(self as *mut _) };
        self.trait_node = Node::new(self_ref);
        &self.trait_node
    }
}

impl<T> Workable for WorkItem<T> {
    fn process(&self) {
        (self.function)(&self.data);
    }

    fn release(&self) {
        unsafe {
            ptr::drop_in_place(self.owner.as_ptr())
        }
    }
}

impl<T> Deref for WorkItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for WorkItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct Workqueue {
    _process: &'static ProcessInternal,
    work: Queue<&'static dyn Workable>,
    event_id: Cell<usize>,
    ref_count: AtomicU8,
}

impl Workqueue {
    pub fn new(context: &process::Context) -> WorkqueueBuilder {
        WorkqueueBuilder {
            context,
            stack: None,
            priority: Default::default(),
        }
    }

    pub fn submit<T: 'static>(&self, work: ConstBox<WorkItem<T>>, function: fn(&T)) -> Result<(), AllocError> {
        let mut item = ConstBox::leak(work);
        unsafe {
            (*item.as_mut()).owner = item;
            (*item.as_mut()).function = function;
        }
        let trait_node = unsafe { (*item.as_mut()).trait_node() };
        unsafe {
            self.work.push_back(Box::from_raw(NonNull::new_unchecked(trait_node as *const _ as *mut _)));
        }

        log::trace!("Submitting work to queue.");
        syscall::event_fire(self.event_id.get());
        Ok(())
    }

    // Userland barrier ////////////////////////////////////////////////////////
    fn work(&self) {
        loop {
            syscall::event_await(self.event_id.get(), u32::MAX).ok();

            while let Some(work) = self.work.try_pop_front() {
                work.process();
                work.release();
                Box::leak(work);
            }
        }
    }
}

// Note(unsafe):
unsafe impl Sync for Workqueue { }

pub struct WorkqueueBuilder<'a> {
    context: &'a process::Context,
    stack: Option<Stack>,
    /// Woker priority.
    priority: Priority,
}

impl<'a> WorkqueueBuilder<'a> {
    /// Set worker stack.
    pub fn stack(&mut self, stack: Stack) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    /// Set worker priority.
    pub fn priority(&mut self, priority: Priority) -> &mut Self {
        self.priority = priority;
        self
    }

    pub fn build(&mut self) -> WorkqueueHandle {
        let id = syscall::event_register();
        assert_ne!(id, 0);

        let stack = match self.stack.take() {
            Some(s) => s,
            None => panic!("No stack added to worker."),
        };

        let worker =
            Box::try_new_in(Workqueue {
                _process: self.context.process(),
                work: Queue::new(),
                event_id: Cell::new(id),
                ref_count: Default::default(),
            }, self.context.process().allocator());
        let worker = match worker {
            Ok(w) => w,
            Err(_) => panic!("No memory left."),
        };
        let worker_handle = WorkqueueHandle::from(worker);

        let thread_handle = worker_handle.clone();
        Thread::new(self.context)
            .priority(self.priority)
            .stack(stack)
            .spawn(move || thread_handle.work());

        worker_handle
    }
}

pub struct WorkqueueHandle {
    workqueue: NonNull<Workqueue>,
}

impl WorkqueueHandle {
    pub fn new(workqueue: NonNull<Workqueue>) -> Self {
        unsafe { workqueue.as_ref() }.ref_count.fetch_add(1, Ordering::Relaxed);
        WorkqueueHandle {
            workqueue,
        }
    }
}

impl Deref for WorkqueueHandle {
    type Target = Workqueue;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.workqueue.as_ref()) }
    }
}

impl From<Box<Workqueue>> for WorkqueueHandle {
    fn from(boxed: Box<Workqueue>) -> Self {
        WorkqueueHandle::new(Box::leak(boxed))
    }
}

impl Clone for WorkqueueHandle {
    fn clone(&self) -> Self {
        WorkqueueHandle::new(self.workqueue)
    }
}