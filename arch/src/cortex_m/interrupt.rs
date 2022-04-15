use cortex_m_rt::exception;

// `kernel_interruot_handler` must be implemented by the kernel.
extern {
    fn kernel_interrupt_handler(irqn: u16);
}

#[allow(non_snake_case)]
#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    kernel_interrupt_handler(irqn as u16);
}