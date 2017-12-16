#![no_std]
#![feature(const_size_of)]
#![feature(unique)]

extern crate multiboot2;
extern crate spin;
extern crate x86_64 as extern_x86_64;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate console;

pub mod bitmap_frame_allocator;
pub use bitmap_frame_allocator::*;

pub mod x86_64;
pub use x86_64::paging::remap_the_kernel;

use x86_64::paging::PhysicalAddress;
use spin::Mutex;
use multiboot2::MemoryAreaIter;

pub const PAGE_SIZE: usize = 4096;

static ALLOCATOR: Mutex<Option<BitmapFrameAllocator>> = Mutex::new(None);

/// Init memory module
/// Must be called once, and only once,
pub unsafe fn init(kernel_start: usize, kernel_end: usize, 
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

    pub fn containing_address(address: usize) -> Frame {
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
