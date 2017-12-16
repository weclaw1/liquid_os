#![feature(lang_items)]
#![no_std]

#![feature(compiler_builtins_lib)]
#![feature(const_size_of)]

extern crate spin;
extern crate compiler_builtins;
extern crate multiboot2;
extern crate x86_64;

/// External functions
pub mod externs;

#[macro_use]
extern crate console;
extern crate memory;

mod kernel;


#[no_mangle]
pub extern fn kmain(multiboot_information_address: usize) {
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    kernel::memory::print_memory_areas(memory_map_tag);

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    
    //kernel::memory::print_kernel_sections(elf_sections_tag);

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    let multiboot_start = boot_info.start_address();
    let multiboot_end = boot_info.end_address();

    println!("kernel start: 0x{:x}, kernel end: 0x{:x}", kernel_start, kernel_end);
    println!("multiboot start: 0x{:x}, multiboot end: 0x{:x}", multiboot_start, multiboot_end);

    unsafe {memory::init(kernel_start as usize, kernel_end as usize, multiboot_start, multiboot_end, memory_map_tag.memory_areas());}

    kernel::memory::enable_nxe_bit();
    kernel::memory::enable_write_protect_bit();
    memory::remap_the_kernel(boot_info);
    println!("It did not crash!");

    loop{

    }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\nPANIC in {} at line {}:", file, line);
    println!("{}", fmt);
    loop{}
}