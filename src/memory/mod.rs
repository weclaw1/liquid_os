pub mod paging;
pub mod heap_allocator;

mod bitmap_frame_allocator;
mod stack_allocator;

use self::bitmap_frame_allocator::BitmapFrameAllocator;

use self::paging::{PAGE_SIZE, PhysicalAddress, Page, ActivePageTable};

use self::heap_allocator::{HEAP_START, HEAP_SIZE};

use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::control::{Cr0, Cr0Flags};

use spin::Mutex;

use multiboot2::{MemoryAreaIter, ElfSectionsTag, MemoryMapTag, BootInformation};

pub use self::stack_allocator::Stack;

use self::stack_allocator::StackAllocator;

const STACK_ALLOCATOR_PAGES: usize = 100;

static ALLOCATOR: Mutex<Option<BitmapFrameAllocator>> = Mutex::new(None);

/// Init memory allocator
/// Must be called once, and only once,
pub unsafe fn frame_allocator_init(kernel_start: usize, kernel_end: usize, 
                   multiboot_start: usize, multiboot_end: usize, 
                   memory_areas: MemoryAreaIter) {
    *ALLOCATOR.lock() = Some(BitmapFrameAllocator::new(&mut bitmap_frame_allocator::BITMAP, 
                             kernel_start, kernel_end, multiboot_start, multiboot_end, memory_areas));
}

pub fn allocate_frame() -> Option<Frame> {
    if let Some(ref mut allocator) = *ALLOCATOR.lock() {
        allocator.allocate_frame()
    } else {
        panic!("frame allocator not initialized");
    }
}

pub fn deallocate_frame(frame: Frame) {
    if let Some(ref mut allocator) = *ALLOCATOR.lock() {
        allocator.deallocate_frame(frame)
    } else {
        panic!("frame allocator not initialized");
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    number: usize,
}

impl Frame {
    pub fn number(&self) -> usize {
        self.number
    }

    pub fn containing_address(address: PhysicalAddress) -> Frame {
        Frame{ number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }

    fn clone(&self) -> Frame {
        Frame {
            number: self.number
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct MemoryController {
    active_table: ActivePageTable,
    stack_allocator: StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        self.stack_allocator.alloc_stack(&mut self.active_table, size_in_pages)
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub fn enable_nxe_bit() {
    unsafe { 
        let mut flags = Efer::read();
        flags.insert(EferFlags::NO_EXECUTE_ENABLE);
        Efer::write(flags); 
    }
}

pub fn enable_write_protect_bit() {
    unsafe { 
        let mut flags = Cr0::read();
        flags.insert(Cr0Flags::WRITE_PROTECT);
        Cr0::write(flags); 
    }
}

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

pub fn init(boot_info: &BootInformation) -> MemoryController {
    let memory_map_tag = boot_info.memory_map_tag().expect(
        "Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect(
        "Elf sections tag required");

    let kernel_start = elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections()
        .filter(|s| s.is_allocated()).map(|s| s.addr + s.size).max()
        .unwrap();

    println!("kernel start: {:#x}, kernel end: {:#x}",
             kernel_start,
             kernel_end);
    println!("multiboot start: {:#x}, multiboot end: {:#x}",
             boot_info.start_address(),
             boot_info.end_address());

    unsafe {frame_allocator_init(kernel_start as usize, kernel_end as usize, boot_info.start_address(), boot_info.end_address(), memory_map_tag.memory_areas());}

    let mut active_table = paging::remap_the_kernel(boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        let result = active_table.map(page, paging::EntryFlags::WRITABLE);
        result.flush(&mut active_table);
    }

    unsafe {heap_allocator::init(HEAP_START, HEAP_SIZE);}

    let stack_allocator = {
        let stack_alloc_start = heap_end_page + 1;
        let stack_alloc_end = stack_alloc_start + STACK_ALLOCATOR_PAGES;
        let stack_alloc_range = Page::range_inclusive(stack_alloc_start,
                                                      stack_alloc_end);
        StackAllocator::new(stack_alloc_range)
    };

    MemoryController {
        active_table: active_table,
        stack_allocator: stack_allocator,
    }

}