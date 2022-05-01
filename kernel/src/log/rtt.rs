#[macro_export]
macro_rules!trace {
    ($ ($args: tt)*) => {
        {
            $crate::log::rtt_target::rprint!("TRACE ");
            $crate::log::rtt_target::rprintln!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!debug {
    ($ ($args: tt)*) => {
        {
            $crate::log::rtt_target::rprint!("DEBUG ");
            $crate::log::rtt_target::rprintln!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!info {
    ($ ($args: tt)*) => {
        {
            $crate::log::rtt_target::rprint!("INFO  ");
            $crate::log::rtt_target::rprintln!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!warn {
    ($ ($args: tt)*) => {
        {
            $crate::log::rtt_target::rprint!("WARN  ");
            $crate::log::rtt_target::rprintln!($($args)*);
        }
    }
}

#[macro_export]
macro_rules!error {
    ($ ($args: tt)*) => {
        {
            $crate::log::rtt_target::rprint!("ERROR ");
            $crate::log::rtt_target::rprintln!($($args)*);
        }
    }
}
