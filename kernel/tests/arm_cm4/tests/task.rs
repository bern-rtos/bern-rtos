#![no_main]
#![no_std]

mod common;
use common::main as _;

#[bern_test::tests]
mod tests {
    use crate::common::Board;
    use stm32f4xx_hal::prelude::*;
    use bern_kernel as kernel;
    use kernel::sched;
    use kernel::task::{Task, Priority};

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        sched::set_tick_frequency(
            1_000,
            48_000_000
        );

        /* idle task */
        Task::new()
            .idle_task()
            .static_stack(kernel::alloc_static_stack!(128))
            .spawn(move || {
                loop {
                    cortex_m::asm::nop();
                }
            });
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
        Task::new()
            .static_stack(kernel::alloc_static_stack!(512))
            .spawn(move || {
                loop {
                    led.toggle().ok();
                    kernel::sleep(100);
                }
            });

        /* watchdog */
        Task::new()
            .priority(Priority(0))
            .static_stack(kernel::alloc_static_stack!(512))
            .spawn(move || {
                kernel::sleep(1000);

                /* if the test does not fail within x time it succeeded */
                bern_test::test_succeeded();
                __tear_down();
            });
        sched::start();
    }
}