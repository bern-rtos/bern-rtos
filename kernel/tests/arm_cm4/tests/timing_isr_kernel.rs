#![no_main]
#![no_std]
#![feature(default_alloc_error_handler)]

use crate::common_timing::{Board, _stm32f4xx_hal_gpio_ExtiPin};
use bern_kernel::exec::interrupt::{InterruptHandler, InterruptStack};
use bern_kernel::exec::process::Context;
use bern_kernel::exec::runnable::Priority;
use bern_kernel::exec::thread::Thread;
use bern_kernel::stack::Stack;
use core::sync::atomic::{compiler_fence, Ordering};

mod common_timing;

// For the direct latency test the interrupt is handled in kernel mode,
// bypassing context switches. This one still uses the kernel interrupt handler.
pub fn spawn_timing_thread(c: &Context, mut board: Board) {
    board.enable_interrupts();
    let mut input = board.shield.button_0;
    let mut output = board.shield.led_7;

    InterruptHandler::new(c)
        .connect_interrupt(st_nucleo_f446::hal::interrupt::EXTI9_5 as u16)
        .stack(InterruptStack::Kernel)
        .handler(move |_c| {
            output.set_high();
            output.set_low();
            input.clear_interrupt_pending_bit();
        });

    Thread::new(c)
        .priority(Priority::new(0))
        .stack(Stack::try_new_in(c, 1024).unwrap())
        .spawn(move || loop {
            compiler_fence(Ordering::SeqCst);
        });
}
