use crate::exec::interrupt::InterruptStack::Kernel;
use crate::exec::process::{self, Process};
use crate::mem::boxed::Box;

pub struct InterruptHandler {
    process: &'static Process,
    handler: &'static mut dyn FnMut(&Context),
    stack: InterruptStack,
    irqn: [Option<u16>; 16],
}

impl InterruptHandler {
    pub fn new(context: &process::Context) -> InterruptBuilder {
        InterruptBuilder::new(context.process())
    }

    pub(crate) fn interrupts(&self) -> &[Option<u16>; 16] {
        &self.irqn
    }

    pub(crate) fn contains_interrupt(&self, irqn: u16) -> bool {
        for i in self.irqn.iter() {
            if *i == Some(irqn) {
                return true;
            }
        }

        false
    }

    pub(crate) fn call(&mut self, irqn: u16) {
        (self.handler)(&Context::new(irqn));
    }
}

pub struct Context {
    irqn: u16,
}

impl Context {
    fn new(irqn: u16) -> Context {
        Context {
            irqn
        }
    }

    pub fn irqn(&self) -> u16 {
        self.irqn
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum InterruptStack {
    Kernel,
    //Process,
}

pub struct InterruptBuilder {
    process: &'static Process,
    stack: Option<InterruptStack>,
    irqn: [Option<u16>; 16],
}

impl InterruptBuilder {
    fn new(process: &'static Process) -> Self {
        InterruptBuilder {
            process,
            stack: None,
            irqn: [None; 16],
        }
    }

    pub fn stack(&mut self, stack: InterruptStack) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    pub fn connect_interrupt(&mut self, irqn: u16) -> &mut Self {
        for i in self.irqn.iter_mut() {
            if i.is_none() {
                *i = Some(irqn);
                break;
            }
        }

        self
    }

    pub fn handler<F>(&mut self, handler: F)
        where F: 'static + FnMut(&Context)
    {
        if self.stack == Some(Kernel) {
            let mut boxed_handler = match Box::try_new_in(handler, self.process.allocator()) {
                Ok(b) => b,
                Err(_) => { return; }
            };

            // todo: introduce lifetime and deallocation
            let handler_runnable = &mut *boxed_handler as *mut _;
            Box::leak(boxed_handler);
            self.build(unsafe { &mut *handler_runnable });
        }
    }


    pub(crate) fn build(&mut self, handler: &'static mut dyn FnMut(&Context)) {
        // Note(unsafe): The stack type is checked before this function call.
        let interrupt = InterruptHandler {
            process: self.process,
            handler,
            stack: unsafe { self.stack.unwrap_unchecked() },
            irqn: self.irqn,
        };
            
        crate::sched::interrupt_handler_add(interrupt);
    }
}
