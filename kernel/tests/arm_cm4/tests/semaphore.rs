#![no_main]
#![no_std]

mod common;
use common::main as _;
use bern_kernel::sync::Semaphore;

#[link_section = ".shared"]
static SEMAPHORE: Semaphore = Semaphore::new(1);

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
        /* idle task */
        Task::new()
            .idle_task()
            .static_stack(kernel::alloc_static_stack!(128))
            .spawn(move || {
                loop {
                    cortex_m::asm::nop();
                }
            });

        /* watchdog */
        Task::new()
            .priority(Priority(0))
            .static_stack(kernel::alloc_static_stack!(512))
            .spawn(move || {
                kernel::sleep(100); // todo: replace with event

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
    fn not_registered() {
        Task::new()
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match SEMAPHORE.acquire(1000) {
                    Ok(_) => panic!("Semaphore wasn't registered, should have failed"),
                    Err(_) => (),
                }
            });

        sched::start();
    }

    #[test]
    fn wait_for_permit(_board: &mut Board) {
        SEMAPHORE.register().ok();
        /* Taking a permit and blocking it */
        Task::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match SEMAPHORE.try_acquire() {
                    Ok(_) => {
                        assert_eq!(SEMAPHORE.available_permits(), 0);
                        kernel::sleep(10);
                    },
                    Err(_) => panic!("Could not acquire semaphore"),
                }
            });
        /* Wait for permit */
        Task::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || {
                match SEMAPHORE.acquire(1000) {
                    Ok(_) => {
                        assert_eq!(SEMAPHORE.available_permits(), 0);
                    },
                    Err(_) => panic!("Did not wait for semaphore"),
                }
            });

        sched::start();
    }
}