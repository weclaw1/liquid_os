#![feature(lang_items)]
#![no_std]

#![feature(compiler_builtins_lib)]
#![feature(const_size_of)]

extern crate compiler_builtins;
extern crate spin;

/// External functions
pub mod externs;

extern crate multiboot2;

extern crate vga_console;
use vga_console::color::{Color, ColorCode};

#[macro_use]
mod kernel;
//extern crate rlibc;


#[no_mangle]
pub extern fn kmain(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page
    let console = kernel::Console::new();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    kprintln!(console, "memory areas:");
    for area in memory_map_tag.memory_areas() {
        kprintln!(console, "    start: 0x{:x}, end: 0x{:x}, length: 0x{:x}", area.base_addr, area.base_addr + area.length, area.length);
    }

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    
    println!("kernel sections:");

    loop{

    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let console = kernel::Console::new();
    kprintln!(console, "\nPANIC in {} at line {}:", file, line);
    kprintln!(console, "{}", fmt);
    loop{}
}