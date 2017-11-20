use memory::bitmap_frame_allocator;
use memory::bitmap_frame_allocator::BitmapFrameAllocator;
use multiboot2::{MemoryAreaIter, MemoryMapTag, ElfSectionsTag};
use memory::FrameAllocator;
use kernel::console::Console;

//use kernel::kprint;

use spin::Mutex;

pub struct MemoryFrameAllocator {
    pub mem_allocator: Mutex<BitmapFrameAllocator<'static>>,
}

impl MemoryFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, 
               multiboot_start: usize, multiboot_end: usize, 
               memory_areas: MemoryAreaIter) -> MemoryFrameAllocator {
        MemoryFrameAllocator {
            mem_allocator: Mutex::new(BitmapFrameAllocator::new(unsafe {&mut bitmap_frame_allocator::BITMAP}, 
                                        kernel_start, kernel_end, multiboot_start, multiboot_end, memory_areas)),
        }
    }
}

pub fn print_memory_areas(console: &Console, memory_map_tag: &MemoryMapTag) {
    kprintln!(console, "memory areas:");
    for area in memory_map_tag.memory_areas() {
        kprintln!(console, "  start: 0x{:x}, end: 0x{:x}, length: 0x{:x}", 
                              area.base_addr, area.base_addr + area.length, area.length);
    }
}

pub fn print_kernel_sections(console: &Console, elf_sections_tag: &'static ElfSectionsTag) {
    kprintln!(console, "kernel sections:");
    for section in elf_sections_tag.sections() {
        kprintln!(console, "  addr: 0x{:x}, end_addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", 
                              section.addr, section.addr + section.size, section.size, section.flags);
    }
}