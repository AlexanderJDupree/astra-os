// serial.rs - Defines UART16550 Serial Port HAL


use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

////////////////////////////////
// Statics/Constants
////////////////////////////////
 
/*
 * 
 *  |-----------------------|
 *  | COM Port  | IO Port   |
 *  |-----------------------|
 *  |   COM1    |   0x3F8   |
 *  |   COM2    |   0x2F8   |
 *  |   COM3    |   0x3E8   |
 *  |   COM4    |   0x2E8   |
 *  |-----------------------|
 * 
 */

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}


////////////////////////////////
// Macros
////////////////////////////////
 
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Serial write failure");
}

// Print to host through serial interface
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}


// Print to host through serial interface and append a newline
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}


////////////////////////////////
// Tests
////////////////////////////////
 