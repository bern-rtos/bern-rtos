
#[macro_export]
macro_rules! send_system_desc_app_name {
    ($name:literal) => {
        $crate::SystemView::send_system_description(concat!("N=", $name, "\0"))
    }
}

#[macro_export]
macro_rules! send_system_desc_os {
    ($name:literal) => {
        $crate::SystemView::send_system_description(concat!("O=", $name, "\0"))
    }
}

#[macro_export]
macro_rules! send_system_desc_device {
    ($name:literal) => {
        $crate::SystemView::send_system_description(concat!("D=", $name, "\0"))
    }
}

#[macro_export]
macro_rules! send_system_desc_core {
    ($name:literal) => {
        $crate::SystemView::send_system_description(concat!("C=", $name, "\0"))
    }
}

#[macro_export]
macro_rules! send_system_desc_interrupt {
    ($irq_number:literal, $name:literal) => {
        $crate::SystemView::send_system_description(concat!("I#", $irq_number, "=", $name, "\0"))
    }
}