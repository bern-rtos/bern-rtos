//! Generic event system.

use crate::mem::linked_list::LinkedList;
use crate::task::Task;

/// Event errors.
#[allow(dead_code)]
#[repr(u8)]
pub enum Error {
    /// Time out reached while pending for an event
    TimeOut,
    /// No event matches the ID
    InvalidId,
}

/// Event wake strategy.
#[allow(dead_code)]
pub enum Wake {
    /// Wake first from pendable list
    WakeFirst,
    /// Wake all from pendable list
    WakeAll,
}

/// Generic event system.
#[allow(dead_code)]
pub struct Event {
    /// Event identifier (randomize to protect access)
    id: usize,
    /// Tasks waiting for the event
    pub pending: LinkedList<Task, super::TaskPool>,
    /// Wake strategy on event
    wake: Wake,
    /// Apply priority inversion
    priority_inversion: bool,
}

impl Event {
    /// Allocate a new event from a given unique ID.
    pub fn new(id: usize) -> Self {
        unsafe {
            Event {
                id,
                pending: LinkedList::new(&*super::TASK_POOL.as_mut_ptr()),
                wake: Wake::WakeFirst,
                priority_inversion: false,
            }
        }
    }

    /// Get the ID of this event.
    pub fn id(&self) -> usize {
        self.id
    }
}