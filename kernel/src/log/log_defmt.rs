#[macro_export]
macro_rules!trace {
    ($ ($args: tt)*) => {
        {
            $crate::log::defmt::trace!($($args)*)
        };
    }
}

#[macro_export]
macro_rules!debug {
    ($ ($args: tt)*) => {
        {
            $crate::log::defmt::debug!($($args)*)
        };
    }
}

#[macro_export]
macro_rules!info {
    ($ ($args: tt)*) => {
        {
            $crate::log::defmt::info!($($args)*)
        };
    }
}

#[macro_export]
macro_rules!warn {
    ($ ($args: tt)*) => {
        {
            $crate::log::defmt::warn!($($args)*)
        };
    }
}

#[macro_export]
macro_rules!error {
    ($ ($args: tt)*) => {
        {
            $crate::log::defmt::error!($($args)*)
        };
    }
}
