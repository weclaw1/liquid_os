use vga_console::{VgaConsole, Buffer};

use spin::Mutex;

#[macro_use]
pub mod kprint;

pub struct Console {
    pub vga: Mutex<VgaConsole>,
}

impl Console {
    pub fn new() -> Console {
        Console {
            vga: Mutex::new(VgaConsole::new(0xb8000 as *mut Buffer)),
        }
    }
}