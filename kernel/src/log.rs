#[cfg(feature = "log-defmt")]
pub mod defmt;

#[cfg(all(not(feature = "log-defmt")))]
mod stub;

pub use {crate::trace, crate::info, crate::warn, crate::error};
