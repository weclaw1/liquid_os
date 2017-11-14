use core::fmt;
use kernel::console::Console;

#[macro_export]
macro_rules! kprintln {
    ($con:ident, $fmt:expr) => (kprint!($con, concat!($fmt, "\n")));
    ($con:ident, $fmt:expr, $($arg:tt)*) => (kprint!($con, concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! kprint {
    ($con:ident, $($arg:tt)*) => ({
        $crate::kernel::kprint::print(&$con, format_args!($($arg)*));
    });
}

pub fn print(con: &Console, args: fmt::Arguments) {
    use core::fmt::Write;
    con.vga.lock().write_fmt(args).unwrap();
}
