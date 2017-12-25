use core::fmt;
use core::fmt::Write;

use spin::Mutex;

pub mod vga_console;

use self::vga_console::{VgaConsole, Buffer};

pub static WRITER: Mutex<VgaConsole> = Mutex::new(VgaConsole::new(0xb8000 as *mut Buffer));

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::drivers::video::console::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}