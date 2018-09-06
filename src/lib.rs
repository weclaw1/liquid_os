#![feature(lang_items)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(abi_x86_interrupt)]
#![feature(panic_handler)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports, unused_attributes))]

#[cfg(test)]
extern crate std;

extern crate spin;

extern crate multiboot2;
extern crate x86_64;
extern crate volatile;
extern crate bit_field;
extern crate slab_allocator;
extern crate uart_16550;

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

/// Drivers
#[macro_use]
mod drivers;

/// Memory management
mod memory;

use memory::heap_allocator;

use core::panic::PanicInfo;

mod interrupts;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: heap_allocator::Allocator = heap_allocator::Allocator;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start(multiboot_information_address: usize) -> ! {
    let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
    let _memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    //memory::print_memory_areas(memory_map_tag);

    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    
    memory::print_kernel_sections(elf_sections_tag);

    memory::enable_nxe_bit();
    memory::enable_write_protect_bit();

    // set up guard page and map the heap pages
    let mut memory_controller = memory::init(boot_info);

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

    drivers::configure();

    // initialize our IDT
    interrupts::init(&mut memory_controller);

    // invoke a breakpoint exception
    //x86_64::instructions::interrupts::int3();

    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
    }

    //stack_overflow();

    println!("It did not crash!");
    serial_println!("Hello Host{}", "!");

    loop{

    }
}

#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
pub extern "C" fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop{}
}