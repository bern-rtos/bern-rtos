#![no_main]
#![no_std]

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
    use bern_kernel::sched;
    use bern_kernel::exec::task::Priority;

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        sched::set_tick_frequency(
            1_000,
            48_000_000
        );

        /* idle task */
        IDLE_PROC.init(move |c| {
            c.new_thread()
                .idle_task()
                .stack(512)
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
            c.new_thread()
                .stack(1024)
                .spawn(move || {
                    loop {
                        led.toggle().ok();
                        kernel::sleep(100);
                    }
                });

            /* watchdog */
            c.new_thread()
                .priority(Priority(0))
                .stack(1024)
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