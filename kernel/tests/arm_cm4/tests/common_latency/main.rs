use core::sync::atomic;
use core::sync::atomic::Ordering;
use super::Board;
use bern_kernel::exec::process::Process;
use bern_kernel::exec::thread::Thread;
use bern_kernel::sched;
use bern_kernel::stack::Stack;
use bern_kernel::bern_arch::cortex_m::cortex_m_rt;

static PROC: &Process = bern_kernel::new_process!(test, 4096);
static IDLE_PROC: &Process = bern_kernel::new_process!(idle, 1024);

#[no_mangle]
#[link_section=".process.test"]
static _DUMMY: u8 = 42;

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::new();

    sched::init();
    sched::set_tick_frequency(
        1_000,
        72_000_000
    );

    // Idle thread
    IDLE_PROC.init(move |c| {
        Thread::new(c)
            .idle_task()
            .stack(Stack::try_new_in(c, 512).unwrap())
            .spawn(move || {
                loop {}
            });
    }).ok();

    PROC.init(move |c| {
        crate::spawn_interrupt_thread(c, board);
    }).ok();

    defmt::info!("Starting interrupt latency test application.");
    bern_kernel::kernel::start();
}

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("Application panicked!");
    defmt::error!("{}", defmt::Display2Format(info));

    cortex_m::asm::bkpt();

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}