#[cfg(target_os = "none")]
pub mod defmt;

#[cfg(not(target_os = "none"))]
mod stub;

pub use {crate::trace, crate::info, crate::warn, crate::error};
