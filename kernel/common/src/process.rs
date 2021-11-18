
pub struct ProcessMemory {
    pub size: usize,

    pub bss_start: *const u8,
    pub bss_end: *const u8,
    pub bss_load: *const u8,

    pub heap_start: *const u8,
    pub heap_end: *const u8,
}