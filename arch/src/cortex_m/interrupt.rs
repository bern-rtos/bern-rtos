use cortex_m_rt::exception;

// `kernel_interruot_handler` must be implemented by the kernel.
extern {
    fn kernel_interrupt_handler(irqn: u16);
}

#[allow(non_snake_case)]
#[exception]
fn DefaultHandler(irqn: i16) {
    unsafe { kernel_interrupt_handler(irqn as u16); }
}