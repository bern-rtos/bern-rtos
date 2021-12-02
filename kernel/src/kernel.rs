use core::cell::Cell;
use crate::process::Process;
use crate::sched;

pub(crate) mod static_memory;


#[link_section = ".kernel"]
pub(crate) static KERNEL: Kernel = Kernel::new();

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Startup,
    Running,
}

pub struct Kernel {
    /// Kernel state.
    state: Cell<State>,
    /// Currently initializing process.
    init_process: Cell<Option<&'static Process>>,
}

impl Kernel {
    pub(crate) const fn new() -> Self {
        Kernel {
            state: Cell::new(State::Startup),
            init_process: Cell::new(None),
        }
    }

    pub(crate) fn start(&self) -> ! {
        self.state.replace(State::Running);

        sched::start();
    }
    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    pub(crate) fn start_init_process(&self, process: &'static Process) {
        self.init_process.replace(Some(process));
    }
    pub(crate) fn end_init_process(&self) {
        self.init_process.replace(None);
    }
    pub(crate) fn process(&self) -> Option<&Process> {
        self.init_process.get()
    }
}


// Note(unsafe): Values within `KERNEL` are only changed at startup, this
// guarantees non-reentrant/single thread operation.
unsafe impl Sync for Kernel { }

pub fn start() -> ! {
    KERNEL.start();
}