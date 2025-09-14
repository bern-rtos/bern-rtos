#[cfg(feature = "log-defmt")]
pub use defmt;
#[cfg(feature = "log-defmt")]
pub mod log_defmt;

#[cfg(feature = "log-rtt")]
pub use rtt_target;
#[cfg(feature = "log-rtt")]
pub mod rtt;

#[cfg(feature = "log-global")]
pub use log::{debug, error, info, trace, warn};

#[cfg(all(
    not(feature = "log-defmt"),
    not(feature = "log-rtt"),
    not(feature = "log")
))]
mod stub;

#[cfg(all(feature = "log-defmt", feature = "log-rtt"))]
compile_error!("Only one log backend can selected");

#[cfg(not(feature = "log-global"))]
pub use {crate::debug, crate::error, crate::info, crate::trace, crate::warn};
