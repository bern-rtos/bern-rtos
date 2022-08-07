mod main;

pub use st_nucleo_f446::StNucleoF446 as Board;
pub use st_nucleo_f446::hal::prelude::*;
pub use st_nucleo_f446::trace::TracePin;

use defmt_rtt as _;