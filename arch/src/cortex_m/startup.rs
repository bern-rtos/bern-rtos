//! ARM Cortex-M implementation of [`IStartup`].

use crate::IStartup;
use crate::arch::Arch;
use r0;
use crate::startup::Region;

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

            r0::init_data(start, end, data);
        }
    }
}