use core::mem;

use super::{Frame, FrameAllocator, FrameIter, PAGE_SIZE};
use multiboot2::{MemoryAreaIter, MemoryArea};

const MAX_MEM_SIZE: usize = 4294967296;
const NUM_OF_FRAMES: usize = MAX_MEM_SIZE/PAGE_SIZE;
const BITS_PER_ITEM: usize = mem::size_of::<usize>() * 8;
pub const ARRAY_SIZE: usize = NUM_OF_FRAMES/BITS_PER_ITEM;

pub static mut BITMAP: [usize; ARRAY_SIZE] = [0; ARRAY_SIZE];

pub struct BitmapFrameAllocator<'a> {
    bitmap: &'a mut [usize; ARRAY_SIZE],
    second_scan: bool,
    next_free_frame: Frame,
    last_frame: Frame,
}

impl<'a> FrameAllocator for BitmapFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let result;
        loop{
            match self.next_free_frame == self.last_frame {
                false => {
                    if self.frame_is_used(self.next_free_frame.number()) {
                        self.next_free_frame = Frame{ number: self.next_free_frame.number() + 1 };
                    } else {
                        let frame = self.next_free_frame.clone();
                        self.set_used(frame.number(), true);
                        self.next_free_frame = Frame{ number: frame.number() + 1};
                        result = Some(frame);
                        break;
                    }
                },
                true if !self.second_scan => {
                    self.second_scan = true;
                    self.next_free_frame = Frame{ number: 0 };
                },
                true => {
                    self.second_scan = false;
                    result = None;
                    break;
                }
            }
        }
        result
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        debug_assert!(frame < self.last_frame);
        self.set_used(frame.number(), false);
    }
}

impl<'a> BitmapFrameAllocator<'a> {
    pub fn new(bitmap: &'a mut [usize; ARRAY_SIZE], kernel_start: usize, kernel_end: usize, 
               multiboot_start: usize, multiboot_end: usize, 
               memory_areas: MemoryAreaIter) -> BitmapFrameAllocator 
    {
        let mut allocator = BitmapFrameAllocator {
            bitmap: bitmap,
            second_scan: false,
            next_free_frame: Frame::containing_address(0),
            last_frame: Frame::containing_address(0),
        };

        allocator.map_memory_areas(memory_areas);
        allocator.map_kernel(kernel_start, kernel_end);
        allocator.map_multiboot(multiboot_start, multiboot_end);
        allocator
    }

    fn set_used(&mut self, index: usize, value: bool) {
        if value {
            self.bitmap[index / BITS_PER_ITEM] |= 1usize << (index % BITS_PER_ITEM);
        } else {
            self.bitmap[index / BITS_PER_ITEM] &= !(1usize << (index % BITS_PER_ITEM));
        }
    }

    fn frame_is_used(&self, index: usize) -> bool {
        (self.bitmap[index / BITS_PER_ITEM] & (1usize << (index % BITS_PER_ITEM))) != 0
    }

    fn map_memory_areas(&mut self, memory_areas: MemoryAreaIter) {
        let last_area = memory_areas.clone().max_by_key(|area| area.base_addr).unwrap();
        self.last_frame = Frame::containing_address(last_area.base_addr as usize + last_area.length as usize);


        for (area1, area2) in memory_areas.clone().zip(memory_areas.clone().skip(1)) {
            let start_occupied = Frame::containing_address((area1.base_addr + area1.length) as usize);
            let end_occupied = Frame::containing_address((area2.base_addr - 1) as usize);

            for frame in Frame::range_inclusive(start_occupied, end_occupied) {
                self.set_used(frame.number(), true);
            }
        }
    }

    fn map_kernel(&mut self, kernel_start: usize, kernel_end: usize) {
        for frame in Frame::range_inclusive(Frame::containing_address(kernel_start), 
                                            Frame::containing_address(kernel_end)) {
            self.set_used(frame.number(), true);
        }
    }

    fn map_multiboot(&mut self, multiboot_start: usize, multiboot_end: usize) {
        for frame in Frame::range_inclusive(Frame::containing_address(multiboot_start), 
                                            Frame::containing_address(multiboot_end)) {
            self.set_used(frame.number(), true);
        }
    }
}
