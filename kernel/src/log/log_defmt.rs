#[macro_export]
macro_rules!trace {
    ($ ($args: tt)*) => {
        {
            defmt::trace!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!debug {
    ($ ($args: tt)*) => {
        {
            defmt::debug!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!info {
    ($ ($args: tt)*) => {
        {
            defmt::info!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!warn {
    ($ ($args: tt)*) => {
        {
            defmt::warn!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!error {
    ($ ($args: tt)*) => {
        {
            defmt::error!($($args)*);
        }
    }
}
