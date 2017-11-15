#![feature(lang_items)]
#![no_std]

#![feature(compiler_builtins_lib)]
#![feature(const_size_of)]

extern crate spin;
extern crate compiler_builtins;
extern crate multiboot2;

/// External functions
pub mod externs;

extern crate console;
extern crate memory;

#[macro_use]
mod kernel;

use memory::FrameAllocator;
use kernel::console::Console;

#[no_mangle]
pub extern fn kmain(multiboot_information_address: usize) {
    // ATTENTION: we have a very small stack and no guard page
    let console = Console::new();

    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    kprintln!(console, "memory areas:");
    for area in memory_map_tag.memory_areas() {
        kprintln!(console, "  start: 0x{:x}, end: 0x{:x}, length: 0x{:x}", 
                              area.base_addr, area.base_addr + area.length, area.length);
    }

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    
    // kprintln!(console, "kernel sections:");
    // for section in elf_sections_tag.sections() {
    //     kprintln!(console, "  addr: 0x{:x}, end_addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", 
    //                           section.addr, section.addr + section.size, section.size, section.flags);
    // }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    let multiboot_start = boot_info.start_address();
    let multiboot_end = boot_info.end_address();

    kprintln!(console, "kernel start: 0x{:x}, kernel end: 0x{:x}", kernel_start, kernel_end);
    kprintln!(console, "multiboot start: 0x{:x}, multiboot end: 0x{:x}", multiboot_start, multiboot_end);

    let mut frame_allocator = memory::BitmapFrameAllocator::new(unsafe {&mut memory::bitmap_frame_allocator::BITMAP},
                                                                kernel_start as usize, kernel_end as usize, multiboot_start,
                                                                multiboot_end, memory_map_tag.memory_areas());
    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            kprintln!(console, "allocated {} frames", i);
            break;
        }
    }
    
    loop{

    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    let console = Console::new();
    kprintln!(console, "\nPANIC in {} at line {}:", file, line);
    kprintln!(console, "{}", fmt);
    loop{}
}