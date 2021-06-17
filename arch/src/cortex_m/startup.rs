//! ARM Cortex-M implementation of [`IStartup`].

use crate::IStartup;
use crate::arch::Arch;
use r0;
use crate::startup::Region;

extern "C" {
    static mut __sshared: u32;
    static mut __eshared: u32;
    static __sishared: u32;
}

impl IStartup for Arch {
    fn init_static_memory() {
        unsafe {
            let shared_ptr = &mut __sshared;
            r0::init_data(shared_ptr, &mut __eshared, &__sishared);
        }
    }

    fn region() -> Region {
        unsafe {
            Region {
                start: &__sshared as *const _ as *const usize,
                stop: &__eshared as *const _ as *const usize,
            }
        }
    }
}