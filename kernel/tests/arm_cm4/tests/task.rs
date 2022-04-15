#![no_main]
#![no_std]

#![feature(default_alloc_error_handler)]

mod common;

use bern_kernel::exec::process::Process;
use common::main as _;

static PROC: &Process = bern_kernel::new_process!(test, 4096);
static IDLE_PROC: &Process = bern_kernel::new_process!(idle, 1024);

#[link_section=".process.test"]
static _DUMMY: u8 = 42;

#[bern_test::tests]
mod tests {
    use super::*;
    use crate::common::Board;
    use stm32f4xx_hal::prelude::*;
    use bern_kernel as kernel;
    use bern_kernel::exec::runnable::Priority;
    use bern_kernel::exec::thread::Thread;
    use bern_kernel::sched;
    use bern_kernel::stack::Stack;

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        sched::set_tick_frequency(
            1_000,
            48_000_000
        );

        /* idle task */
        IDLE_PROC.init(move |c| {
            Thread::new(c)
                .idle_task()
                .stack(Stack::try_new_in(c, 512).unwrap())
                .spawn(move || {
                    loop {}
                });
        }).ok();
    }

    #[test_tear_down]
    fn reset() {
        cortex_m::peripheral::SCB::sys_reset();
    }

    #[tear_down]
    fn stop() {
        cortex_m::asm::bkpt();
    }

    #[test]
    fn first_task(board: &mut Board) {
        let mut led = board.led.take().unwrap();

        PROC.init(move |c| {
            Thread::new(c)
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    loop {
                        led.toggle().ok();
                        kernel::sleep(100);
                    }
                });

            /* watchdog */
            Thread::new(c)
                .priority(Priority::new(0))
                .stack(Stack::try_new_in(c, 1024).unwrap())
                .spawn(move || {
                    kernel::sleep(1000);

                    /* if the test does not fail within x time it succeeded */
                    bern_test::test_succeeded();
                    __tear_down();
                });
        }).ok();

        bern_kernel::kernel::start();
    }
}