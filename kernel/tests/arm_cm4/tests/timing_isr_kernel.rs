#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

use bern_kernel::exec::interrupt::{InterruptHandler, InterruptStack};
use bern_kernel::exec::process::Context;
use crate::common_timing::{_stm32f4xx_hal_gpio_ExtiPin, Board};

mod common_timing;

// For the direct latency test the interrupt is handled in kernel mode,
// bypassing context switches. This one still uses the kernel interrupt handler.
pub fn spawn_timing_thread(c: &Context, board: Board) {
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
}