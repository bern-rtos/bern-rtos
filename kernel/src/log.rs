#[cfg(feature = "log-defmt")]
pub use defmt;
#[cfg(feature = "log-defmt")]
pub mod log_defmt;

#[cfg(all(not(feature = "log-defmt")))]
mod stub;

pub use {crate::trace, crate::debug, crate::info, crate::warn, crate::error};
