#[macro_export]
macro_rules ! trace {
    ( $($arg:expr),+ ) => {
        {
            $(let _ = $arg;)+
        }
    }
}

#[macro_export]
macro_rules ! debug {
    ( $($arg:expr),+ ) => {
        {
            $(let _ = $arg;)+
        }
    }
}

#[macro_export]
macro_rules ! info {
    ( $($arg:expr),+ ) => {
        {
            $(let _ = $arg;)+
        }
    }
}

#[macro_export]
macro_rules ! warn {
    ( $($arg:expr),+ ) => {
        {
            $(let _ = $arg;)+
        }
    }
}

#[macro_export]
macro_rules ! error {
    ( $($arg:expr),+ ) => {
        {
            $(let _ = $arg;)+
        }
    }
}