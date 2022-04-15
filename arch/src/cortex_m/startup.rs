//! ARM Cortex-M implementation of [`IStartup`].

use core::arch::asm;
use crate::IStartup;
use crate::arch::Arch;
use crate::startup::Region;

extern "C" {
    static mut __smkernel: usize;
    static mut __emkernel: usize;
    static __sikernel: usize;

    static mut __shkernel: usize;
    static mut __ehkernel: usize;
}

// todo: r0 is deprecated, replace with assembly

impl IStartup for Arch {
    fn init_static_region(mut region: Region) {
        unsafe {
            let mut start = region.start as *mut u32;
            let end = region.end as *mut u32;
            let mut data = match region.data {
                None => return,
                Some(d) => d,
            } as *const u32;

            asm!(
                "movs r3, #0
                  b 1f
                0:
                  ldr r4, [r2, r3]
                  str r4, [r0, r3]
                  adds r3, r3, #4
                1:
                  adds r4, r0, r3
                  cmp r4, r1
                  bcc 0b",
                in("r0") start,
                in("r1") end,
                in("r2") data
            )
        }
    }


    fn kernel_data() -> Region {
        unsafe {
            Region {
                start: &__smkernel as *const _,
                end: &__emkernel as *const _,
                data: Some(&__sikernel as *const _)
            }
        }
    }

    fn kernel_heap() -> Region {
        unsafe {
            Region {
                start: &__shkernel as *const _,
                end: &__ehkernel as *const _,
                data: None
            }
        }
    }
}