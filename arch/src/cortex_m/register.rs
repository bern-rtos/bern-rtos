//! CPU Register definitions.
//!
//! Adapted from the `cortex-m` crate.

/// CPU registers pushed/popped by the hardware
#[repr(C)]
pub struct StackFrame {
    /// (General purpose) Register 0
    pub r0: u32,
    /// (General purpose) Register 1
    pub r1: u32,
    /// (General purpose) Register 2
    pub r2: u32,
    /// (General purpose) Register 3
    pub r3: u32,
    /// (General purpose) Register 12
    pub r12: u32,
    /// Link Register
    pub lr: u32,
    /// Program Counter
    pub pc: u32,
    /// Program Status Register
    pub xpsr: u32,
}

/// CPU registers the software must push/pop to/from the stack
#[repr(C)]
pub struct StackFrameExtension {
    /// (General purpose) Register 4
    pub r4: u32,
    /// (General purpose) Register 5
    pub r5: u32,
    /// (General purpose) Register 6
    pub r6: u32,
    /// (General purpose) Register 7
    pub r7: u32,
    /// (General purpose) Register 8
    pub r8: u32,
    /// (General purpose) Register 9
    pub r9: u32,
    /// (General purpose) Register 10
    pub r10: u32,
    /// (General purpose) Register 11
    pub r11: u32,
}

/// FPU registers the software must push/pop to/from the stack
// todo: CPU registers used by the floating point unit
#[allow(dead_code)]
#[repr(C)]
pub struct StackFrameFpu {
}