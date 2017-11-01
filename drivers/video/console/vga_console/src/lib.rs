#![no_std]

pub mod color;

use color::{Color, ColorCode};

use core::fmt;

extern crate volatile;
use volatile::Volatile;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct VgaConsole {
    buffer: *mut Buffer,
    color_code: ColorCode,
    position: usize,
}

impl VgaConsole {
    pub fn new(buffer: *mut Buffer) -> VgaConsole {
        VgaConsole {
            buffer: buffer,
            color_code: ColorCode::new(Color::Green, Color::Black),
            position: 0,
        }
    }

    fn write_byte(&mut self, byte: u8) {
        let current_line = self.position / BUFFER_WIDTH;
        let current_column = self.position - current_line * BUFFER_WIDTH;
        if byte == b'\n' {
            self.position = (current_line + 1) * BUFFER_WIDTH;
        } else {
            let color_code = self.color_code;
            self.buffer().chars[current_line][current_column].write(
                ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                }
            );

            self.position += 1;
        }

        if self.position >= BUFFER_HEIGHT*BUFFER_WIDTH {
            self.scroll();
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe{ self.buffer.as_mut().unwrap() }
    }

    fn scroll(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for column in 0..BUFFER_WIDTH {
                let character = self.buffer().chars[row][column].read();
                self.buffer().chars[row-1][column].write(character);
            }
        }

        for column in 0..BUFFER_WIDTH {
            self.buffer().chars[BUFFER_HEIGHT-1][column].write(
                ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::Black, Color::Black),
                }
            );
        }

        self.position = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH;
    }

    pub fn clear_screen(&mut self) {
        for row in self.buffer().chars.iter_mut() {
            for character in row.iter_mut() {
                character.write(
                    ScreenChar {
                        ascii_character: b' ',
                        color_code: ColorCode::new(Color::Black, Color::Black),
                    }
                );
            }
        }

        self.position = 0;
    }

    pub fn set_color_code(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

}

impl fmt::Write for VgaConsole {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::mem;
    use core::fmt::Write;
    use super::*;

    #[test]
    fn writing_character_works() {
        let mut vga_mem : [u8; BUFFER_WIDTH * BUFFER_HEIGHT * 2] = [0 ; BUFFER_WIDTH * BUFFER_HEIGHT * 2];
        unsafe {
            let vga_mem_pointer = mem::transmute::<&mut u8, *mut Buffer>(&mut vga_mem[0]);
            let mut vga_console = VgaConsole::new(vga_mem_pointer);
            vga_console.write_byte(b'a');
        } 
        
        assert_eq!(vga_mem[0], b'a');
    }

    #[test]
    fn writing_str_works() {
        let mut vga_mem : [u8; BUFFER_WIDTH * BUFFER_HEIGHT * 2] = [0 ; BUFFER_WIDTH * BUFFER_HEIGHT * 2];
        let mem_after_writing : [u8; 6] = [b'a', 0x02, b'l', 0x02, b'a', 0x02];
        unsafe {
            let vga_mem_pointer = mem::transmute::<&mut u8, *mut Buffer>(&mut vga_mem[0]);
            let mut vga_console = VgaConsole::new(vga_mem_pointer);
            vga_console.set_color_code(ColorCode::new(Color::Green, Color::Black));
            vga_console.write_str("ala");
        } 
        
        assert_eq!(&vga_mem[0..6], &mem_after_writing[0..6]);
    }
}
