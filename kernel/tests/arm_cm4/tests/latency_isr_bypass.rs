//! For the direct latency test the kernel is bypassed altogether.

#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

use bern_kernel::exec::process::Context;
use crate::common_latency::Board;
use st_nucleo_f446::hal;
use hal::interrupt;

mod common_latency;

pub fn spawn_interrupt_thread(_c: &Context, _board: Board) {

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