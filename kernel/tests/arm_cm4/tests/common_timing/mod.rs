mod main;

pub use st_nucleo_f446::hal::prelude::*;
pub use st_nucleo_f446::trace::TracePin;
pub use st_nucleo_f446::StNucleoF446 as Board;

use defmt_rtt as _;
