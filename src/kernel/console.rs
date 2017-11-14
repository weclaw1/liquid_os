use console::vga_console::{VgaConsole, Buffer};

use spin::Mutex;

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