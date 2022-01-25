use core::cell::Cell;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU8, Ordering};
use core::ops::Deref;
use crate::alloc::allocator::AllocError;
use crate::exec::process;
use crate::exec::process::Process;
use crate::exec::runnable::Priority;
use crate::exec::thread::Thread;
use crate::mem::boxed::Box;
use crate::mem::queue::FiFoQueue;
use crate::mem::queue::mpsc_const::ConstQueue;
use crate::stack::Stack;
use crate::sync::channel::RefMessage;
use crate::syscall;

//pub trait WorkTrait: 'static + FnOnce() { }

pub struct ConstWorkqueue<const N: usize> {
    process: &'static Process,
    work: ConstQueue<RefMessage<fn()>, { N }>,
    event_id: Cell<usize>,
    ref_count: AtomicU8,
}

impl<const N: usize> ConstWorkqueue<{ N }> {
    pub fn new(context: &process::Context) -> ConstWorkqueueBuilder<{ N }> {
        ConstWorkqueueBuilder {
            context,
            stack: None,
            priority: Default::default(),
        }
    }

    //pub fn submit<F>(&self, work: F) -> Result<(), AllocError>
    pub fn submit(&self, work: fn()) -> Result<(), AllocError>
        //where F: WorkTrait
    {
        let boxed_work = Box::try_new_in(work, self.process.allocator())?;
        let work_ptr = Box::leak(boxed_work).as_ptr();
        //let entry_ptr = unsafe { &*Box::leak(boxed_work).as_ptr() };
        self.work.try_push_back(RefMessage::from(work_ptr)).unwrap();

        syscall::event_fire(self.event_id.get());
        Ok(())
    }

    // Userland barrier ////////////////////////////////////////////////////////
    fn work(&self) {
        loop {
            syscall::event_await(self.event_id.get(), u32::MAX).ok();

            while let Ok(work_ref) = self.work.try_pop_front() {
                unsafe { (*work_ref.into_mut_ptr())() };
            }
        }
    }
}

// Note(unsafe):
unsafe impl<const N: usize> Sync for ConstWorkqueue<{ N }> { }

pub struct ConstWorkqueueBuilder<'a, const N: usize> {
    context: &'a process::Context,
    stack: Option<Stack>,
    /// Woker priority.
    priority: Priority,
}

impl<'a, const N: usize> ConstWorkqueueBuilder<'a, { N }> {
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

    //pub fn build(&mut self) -> &Workqueue {
    pub fn build(&mut self) -> ConstWorkqueueHandle<{ N }> {
        let id = syscall::event_register();
        assert_ne!(id, 0);

        let stack = match self.stack.take() {
            Some(s) => s,
            None => panic!("No stack added to worker."),
        };

        let worker =
            Box::try_new_in(ConstWorkqueue {
                process: self.context.process(),
                work: ConstQueue::<RefMessage<fn()>, { N }>::new(),
                event_id: Cell::new(id),
                ref_count: Default::default(),
            }, self.context.process().allocator());
        let mut worker = match worker {
            Ok(w) => w,
            Err(_) => panic!("No memory left."),
        };
        let worker_handle = ConstWorkqueueHandle::<{ N }>::from(worker);

        let thread_handle = worker_handle.clone();
        Thread::new(self.context)
            .priority(self.priority)
            .stack(stack)
            .spawn(move || thread_handle.work());

        worker_handle
    }
}

pub struct ConstWorkqueueHandle<const N: usize> {
    workqueue: NonNull<ConstWorkqueue<{ N }>>,
}

impl<const N: usize> ConstWorkqueueHandle<{ N }> {
    pub fn new(workqueue: NonNull<ConstWorkqueue<{ N }>>) -> Self {
        unsafe { workqueue.as_ref() }.ref_count.fetch_add(1, Ordering::Relaxed);
        ConstWorkqueueHandle {
            workqueue,
        }
    }
}

impl<const N: usize> Deref for ConstWorkqueueHandle<{ N }> {
    type Target = ConstWorkqueue<{ N }>;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.workqueue.as_ref()) }
    }
}

impl<const N: usize> From<Box<ConstWorkqueue<{ N }>>> for ConstWorkqueueHandle<{ N }> {
    fn from(boxed: Box<ConstWorkqueue<{ N }>>) -> Self {
        ConstWorkqueueHandle::new(Box::leak(boxed))
    }
}

impl<const N: usize> Clone for ConstWorkqueueHandle<{ N }> {
    fn clone(&self) -> Self {
        ConstWorkqueueHandle::new(self.workqueue)
    }
}