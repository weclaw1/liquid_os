use multiboot2::{MemoryMapTag, ElfSectionsTag};

pub fn print_memory_areas(memory_map_tag: &MemoryMapTag) {
    println!("memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("  start: 0x{:x}, end: 0x{:x}, length: 0x{:x}", 
                    area.base_addr, area.base_addr + area.length, area.length);
    }
}

pub fn print_kernel_sections(elf_sections_tag: &'static ElfSectionsTag) {
    println!("kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("  addr: 0x{:x}, end_addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", 
                            section.addr, section.addr + section.size, section.size, section.flags);
    }
}