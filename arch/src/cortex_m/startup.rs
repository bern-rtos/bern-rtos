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
                "0:
                  cmp r1, r0
                  beq 1f
                  ldm r2!, {{r3}}
                  stm r0!, {{r3}}
                  b   0b
                1:",
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