//! For the direct latency test the kernel is bypassed altogether.

#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

use bern_kernel::exec::process::Context;
use crate::common_timing::Board;
use st_nucleo_f446::hal;
use hal::interrupt;

mod common_timing;

pub fn spawn_timing_thread(_c: &Context, mut board: Board) {
    board.enable_interrupts();
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI9_5() {
    unsafe {
        (*hal::pac::GPIOC::ptr()).odr.modify(|_, w|  w.odr7().set_bit());
        (*hal::pac::EXTI::ptr()).pr.write(|w| w.pr6().set_bit());
        (*hal::pac::GPIOC::ptr()).odr.modify(|_, w|  w.odr7().clear_bit());
    }
}