use super::Board;
use bern_kernel::bern_arch::cortex_m::cortex_m_rt;
use bern_kernel::exec::process::Process;
use bern_kernel::units::frequency::ExtMilliHertz;
use core::sync::atomic;
use core::sync::atomic::Ordering;

static PROC: &Process = bern_kernel::new_process!(test, 4096);

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = Board::new(168);

    bern_kernel::init();
    bern_kernel::time::set_tick_frequency(1.kHz(), 168.MHz());

    PROC.init(move |c| {
        crate::spawn_timing_thread(c, board);
    })
    .ok();

    defmt::info!("Starting interrupt timing test application.");
    bern_kernel::start();
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
