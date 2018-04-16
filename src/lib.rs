#![feature(lang_items)]
#![feature(const_size_of)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(const_fn)]
#![feature(unique)]
#![feature(ptr_internals)]
#![no_std]
#![no_main]

extern crate spin;

extern crate multiboot2;
extern crate x86_64;
extern crate volatile;
extern crate linked_list_allocator;

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate bitflags;

/// External functions
pub mod externs;

/// Drivers
#[macro_use]
mod drivers;

/// Memory management
mod memory;

use memory::heap_allocator;
use memory::heap_allocator::{HEAP_START, HEAP_SIZE};

#[global_allocator]
static ALLOCATOR: heap_allocator::Allocator = heap_allocator::Allocator;

#[no_mangle]
pub extern "C" fn _start(multiboot_information_address: usize) -> ! {
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    //memory::print_memory_areas(memory_map_tag);

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    
    memory::print_kernel_sections(elf_sections_tag);

    memory::enable_nxe_bit();
    memory::enable_write_protect_bit();

    // set up guard page and map the heap pages
    memory::init(boot_info);

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1,2,3,4,5,6,7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    for _ in 0..100000 {
        format!("Some String");
    }


    println!("It did not crash!");

    loop{

    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(msg: core::fmt::Arguments, file: &'static str, line: u32, column: u32) -> ! {
    println!("\nPANIC in {} at line {}:", file, line);
    println!("{}", msg);
    loop{}
}