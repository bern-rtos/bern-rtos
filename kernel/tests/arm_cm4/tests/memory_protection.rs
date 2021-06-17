#![no_main]
#![no_std]

mod common;
use common::main as _;
use bern_kernel::sync::mutex::Mutex;

fn overflow_stack(a: u32) -> u32 {
    overflow_stack(a + 1)
}

struct MyStruct {
    a: u32,
}

#[link_section = ".shared"]
static MUTEX: Mutex<MyStruct> = Mutex::new(MyStruct{ a: 42 });

#[bern_test::tests]
mod tests {
    use super::*;
    use crate::common::*;
    use bern_kernel as kernel;
    use kernel::sched;
    use kernel::task::{Task, Priority};

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        MUTEX.register().ok();
        /* idle task */
        Task::new()
            .idle_task()
            .static_stack(kernel::alloc_static_stack!(512))
            .spawn(move || {
                loop {
                    cortex_m::asm::nop();
                }
            });

        /* watchdog */
        Task::new()
            .priority(Priority(0))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                kernel::sleep(500); // todo: replace with event

                /* if the test does not fail within x time it succeeded */
                bern_test::test_succeeded();
                __test_tear_down();
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
    fn normal_operation() {
        Task::new()
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                // just spawn a task and do nothing special
                kernel::sleep(500);
            });

        sched::start();
    }

    #[test]
    fn gpio_access(board: &mut Board) {
        let mut led = board.led.take().unwrap();
        Task::new()
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                loop {
                    led.toggle().ok();
                    kernel::sleep(10);
                }
            });
        sched::start();
    }


}