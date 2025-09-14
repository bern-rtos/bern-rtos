#![no_main]
#![no_std]

mod common;
use bern_kernel::sync::Semaphore;
use common::main as _;

#[link_section = ".shared"]
static SEMAPHORE: Semaphore = Semaphore::new(1);

#[bern_test::tests]
mod tests {
    use super::*;
    use crate::common::*;
    use bern_kernel as kernel;
    use bern_kernel::exec::thread::{Priority, Runnable, Thread};
    use kernel::sched;

    #[test_set_up]
    fn init_scheduler() {
        sched::init();
        /* watchdog */
        Thread::new()
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
        Thread::new()
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || match SEMAPHORE.acquire(1000) {
                Ok(_) => panic!("Semaphore wasn't registered, should have failed"),
                Err(_) => (),
            });

        sched::start();
    }

    #[test]
    fn wait_for_permit(_board: &mut Board) {
        SEMAPHORE.register().ok();
        /* Taking a permit and blocking it */
        Thread::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || match SEMAPHORE.try_acquire() {
                Ok(_) => {
                    assert_eq!(SEMAPHORE.available_permits(), 0);
                    kernel::sleep(10);
                }
                Err(_) => panic!("Could not acquire semaphore"),
            });
        /* Wait for permit */
        Thread::new()
            .priority(Priority(1))
            .static_stack(kernel::alloc_static_stack!(1024))
            .spawn(move || match SEMAPHORE.acquire(1000) {
                Ok(_) => {
                    assert_eq!(SEMAPHORE.available_permits(), 0);
                }
                Err(_) => panic!("Did not wait for semaphore"),
            });

        sched::start();
    }
}
