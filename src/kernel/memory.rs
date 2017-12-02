use memory::bitmap_frame_allocator;
use memory::bitmap_frame_allocator::BitmapFrameAllocator;
use multiboot2::{MemoryAreaIter, MemoryMapTag, ElfSectionsTag};
use memory::FrameAllocator;
use memory::x86_64::paging;
use kernel::console::Console;

//use kernel::kprint;

use spin::Mutex;

pub struct MemoryFrameAllocator {
    pub allocator: Mutex<BitmapFrameAllocator<'static>>,
}

impl MemoryFrameAllocator {
    pub fn new(kernel_start: usize, kernel_end: usize, 
               multiboot_start: usize, multiboot_end: usize, 
               memory_areas: MemoryAreaIter) -> MemoryFrameAllocator {
        MemoryFrameAllocator {
            allocator: Mutex::new(BitmapFrameAllocator::new(unsafe {&mut bitmap_frame_allocator::BITMAP}, 
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

pub fn test_paging<A>(console: &Console, allocator: &mut A)
    where A: FrameAllocator
{
    let mut page_table = unsafe { paging::ActivePageTable::new() };

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = paging::Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frames");
    kprintln!(console, "None = {:?}, map to {:?}",
         page_table.translate(addr),
         frame);
    page_table.map_to(page, frame, paging::EntryFlags::empty(), allocator);
    kprintln!(console, "Some = {:?}", page_table.translate(addr));
    kprintln!(console, "next free frame: {:?}", allocator.allocate_frame());

    kprintln!(console, "{:#x}", unsafe {
        *(paging::Page::containing_address(addr).start_address() as *const u64)
    });

    page_table.unmap(paging::Page::containing_address(addr), allocator);
    kprintln!(console, "None = {:?}", page_table.translate(addr));

    kprintln!(console, "{:#x}", unsafe {
        *(paging::Page::containing_address(addr).start_address() as *const u64)
    });
}
