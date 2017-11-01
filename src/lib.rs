#![feature(lang_items)]
#![no_std]

#![feature(compiler_builtins_lib)]
#![feature(const_size_of)]

extern crate compiler_builtins;
extern crate spin;

/// External functions
pub mod externs;

extern crate vga_console;
use vga_console::color::{Color, ColorCode};

#[macro_use]
mod kernel;
//extern crate rlibc;


#[no_mangle]
pub extern fn kmain() {
    // ATTENTION: we have a very small stack and no guard page

    let console = kernel::Console::new();
    
    kprintln!(console, "This should be cleared.");
    console.vga.lock().clear_screen();
    kprintln!(console, "{}", { kprintln!(console, "inner"); "outer" });
    kprintln!(console, "Kernel initialized.");
    kprint!(console, "bbbyo");
    kprintln!(console, "hehehehehheh");
    kprintln!(console, "Hello World{}", "!");

    kprintln!(console, "Kernel initialized.");
    console.vga.lock().set_color_code(ColorCode::new(Color::Blue, Color::Black));
    kprintln!(console, "Kernel initialized.");

    loop{
    // kprintln!(console, "Kernel initialized.");
    // kprintln!(console, "Muahahaha");
    // kprint!(console, "bbbyo");
    // kprintln!(console, "hehehehehheh");
    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] #[no_mangle] pub extern fn panic_fmt() -> ! {loop{}}