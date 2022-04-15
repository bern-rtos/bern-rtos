#[macro_export]
macro_rules!trace {
    ($ ($args: tt)*) => {
        {
            // Note(unsafe): ???
            unsafe { defmt::trace!($($args)*); }
        }
    }
}

#[macro_export]
macro_rules!debug {
    ($ ($args: tt)*) => {
        {
            // Note(unsafe): ???
            unsafe { defmt::debug!($($args)*);  }
        }
    }
}

#[macro_export]
macro_rules!info {
    ($ ($args: tt)*) => {
        {
            // Note(unsafe): ???
            unsafe { defmt::info!($($args)*); }
        }
    }
}

#[macro_export]
macro_rules!warn {
    ($ ($args: tt)*) => {
        {
            // Note(unsafe): ???
            unsafe { defmt::warn!($($args)*); }
        }
    }
}

#[macro_export]
macro_rules!error {
    ($ ($args: tt)*) => {
        {
            // Note(unsafe): ???
            unsafe { defmt::error!($($args)*); }
        }
    }
}
