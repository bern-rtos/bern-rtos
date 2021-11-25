use bern_arch::startup::Region;

extern "C" {
    static mut __smkernel: usize;
    static mut __emkernel: usize;
    static __sikernel: usize;

    static mut __shkernel: usize;
    static mut __ehkernel: usize;
}

pub fn kernel_data() -> Region {
    unsafe {
        Region {
            start: &__smkernel as *const _,
            end: &__emkernel as *const _,
            data: Some(&__sikernel as *const _)
        }
    }
}

pub fn kernel_heap() -> Region {
    unsafe {
        Region {
            start: &__shkernel as *const _,
            end: &__ehkernel as *const _,
            data: None
        }
    }
}