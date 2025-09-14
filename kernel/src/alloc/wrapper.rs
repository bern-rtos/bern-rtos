use crate::alloc::allocator::Allocator;
use crate::kernel::{State, KERNEL};
use crate::sched;
use crate::{log, syscall};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

pub struct Wrapper {}

impl Wrapper {
    pub(crate) const fn new() -> Self {
        Wrapper {}
    }

    pub(crate) fn alloc_handler(layout: Layout) -> *mut u8 {
        Self::with_allocator(|a| match a {
            None => 0 as *mut _,
            Some(a) => match a.alloc(layout) {
                Ok(mut addr) => unsafe { &mut addr.as_mut()[0] as *mut _ },
                Err(_) => 0 as *mut _,
            },
        })
    }

    pub(crate) fn dealloc_handler(ptr: *mut u8, layout: Layout) {
        Self::with_allocator(|a| {
            if let Some(alloc) = a {
                unsafe {
                    alloc.dealloc(NonNull::new_unchecked(ptr), layout);
                }
            }
        })
    }

    fn with_allocator<F, R>(f: F) -> R
    where
        F: FnOnce(Option<&dyn Allocator>) -> R,
    {
        match KERNEL.state() {
            // Kernel not started yet -> get allocator from process context.
            State::Startup => match KERNEL.process() {
                None => {
                    log::error!("Cannot allocate memory outside process.");
                    f(None)
                }
                Some(p) => f(Some(p.allocator())),
            },
            // Kernel is already running -> get allocator from active thread
            State::Running => sched::with_callee(|t| f(Some(t.process().allocator()))),
        }
    }
}

unsafe impl GlobalAlloc for Wrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        syscall::alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        syscall::dealloc(ptr, layout)
    }
}
