pub mod x86_64;
pub mod heap_allocator;

mod bitmap_frame_allocator;

use self::bitmap_frame_allocator::BitmapFrameAllocator;

use self::x86_64::paging::{PAGE_SIZE, PhysicalAddress, Page};
pub use self::x86_64::paging;

use self::heap_allocator::{HEAP_START, HEAP_SIZE};

use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};
use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

use spin::Mutex;

use multiboot2::{MemoryAreaIter, ElfSectionsTag, MemoryMapTag, BootInformation};

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
    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

pub fn enable_write_protect_bit() {
    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
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

pub fn init(boot_info: &BootInformation) {
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
}