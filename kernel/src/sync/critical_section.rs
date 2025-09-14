use bern_arch::arch::Arch;
use bern_arch::ISync;

// similar to cortex_m::interrupt::free
#[inline(always)]
pub fn exec<R>(f: impl FnOnce() -> R) -> R {
    Arch::disable_interrupts(0);
    let r = f();
    Arch::enable_interrupts();
    r
}
