use super::{VirtualAddress, PhysicalAddress, Page, ENTRY_COUNT, ActivePageTable};
use super::entry::*;
use super::table::{self, Table, Level4, Level1};
use ::{PAGE_SIZE, Frame, FrameAllocator, allocate_frame, deallocate_frame};
use core::ptr::Unique;
use core::mem;
use extern_x86_64;
use extern_x86_64::instructions::tlb;

/// In order to enforce correct paging operations in the kernel, these types
/// are returned on any mapping operation to get the code involved to specify
/// how it intends to flush changes to a page table
#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlush(Page);

impl MapperFlush {
    /// Create a new page flush promise
    pub fn new(page: Page) -> MapperFlush {
        MapperFlush(page)
    }

    /// Flush this page in the active table
    pub fn flush(self, table: &mut ActivePageTable) {
        table.flush(self.0);
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

/// A flush cannot be dropped, it must be consumed
impl Drop for MapperFlush {
    fn drop(&mut self) {
        panic!("Mapper flush was not utilized");
    }
}

/// To allow for combining multiple flushes into one, we have a way of flushing
/// the active table, which can consume MapperFlush structs
#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct MapperFlushAll(bool);

impl MapperFlushAll {
    /// Create a new promise to flush all mappings
    pub fn new() -> MapperFlushAll {
        MapperFlushAll(false)
    }

    /// Consume a single page flush
    pub fn consume(&mut self, flush: MapperFlush) {
        self.0 = true;
        mem::forget(flush);
    }

    /// Flush the active page table
    pub fn flush(self, table: &mut ActivePageTable) {
        if self.0 {
            table.flush_all();
        }
        mem::forget(self);
    }

    /// Ignore the flush. This is unsafe, and a reason should be provided for use
    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

/// A flush cannot be dropped, it must be consumed
impl Drop for MapperFlushAll {
    fn drop(&mut self) {
        panic!("Mapper flush all was not utilized");
    }
}

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        self.p4().next_table(page.p4_index())
        .and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].pointed_frame())
    }

    pub fn map_to(&mut self, page: Page, frame: Frame, flags: EntryFlags) -> MapperFlush {   
        let p3 = self.p4_mut().next_table_create(page.p4_index());
        let p2 = p3.next_table_create(page.p3_index());
        let p1 = p2.next_table_create(page.p2_index());

        assert!(p1[page.p1_index()].is_unused());

        let current_count = p1.entry_count();
        p1.set_entry_count(current_count + 1);

        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
        MapperFlush::new(page)
    }

    pub fn map(&mut self, page: Page, flags: EntryFlags) -> MapperFlush {
        let frame = allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags)
    }

    pub fn identity_map(&mut self, frame: Frame, flags: EntryFlags) -> MapperFlush {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags)
    }

    pub fn unmap_inner(&mut self, page: &Page, keep_parents: bool) -> Frame {
        assert!(self.translate(page.start_address()).is_some());
        let frame;
        {
            let p1 = self.p4_mut()
                         .next_table_mut(page.p4_index())
                         .and_then(|p3| p3.next_table_mut(page.p3_index()))
                         .and_then(|p2| p2.next_table_mut(page.p2_index()))
                         .expect("mapping code does not support huge pages");
            frame = p1[page.p1_index()].pointed_frame().unwrap();
            p1[page.p1_index()].set_unused();
            if keep_parents {
                return frame;
            }
        }

        let p2_freed = {
            let p2 = self.p4_mut()
                         .next_table_mut(page.p4_index())
                         .and_then(|p3| p3.next_table_mut(page.p3_index()))
                         .unwrap();
            let p1_count = p2.next_table(page.p2_index()).unwrap().entry_count();
            if (p1_count - 1) == 0 {
                let frame = p2[page.p2_index()].pointed_frame().unwrap();
                p2[page.p2_index()].set_unused();
                deallocate_frame(frame);
                true
            } else {
                p2.next_table_mut(page.p2_index()).unwrap().set_entry_count(p1_count - 1);
                false
            }
        };

        if p2_freed {
            let p3_freed = {
                let p3 = self.p4_mut()
                             .next_table_mut(page.p4_index())
                             .unwrap();
                let p2_count = p3.next_table(page.p3_index()).unwrap().entry_count();
                if (p2_count - 1) == 0 {
                    let frame = p3[page.p3_index()].pointed_frame().unwrap();
                    p3[page.p3_index()].set_unused();
                    deallocate_frame(frame);
                    true
                } else {
                    p3.next_table_mut(page.p3_index()).unwrap().set_entry_count(p2_count - 1);
                    false
                }
            };

            if p3_freed {
                let p4 = self.p4_mut();

                let p3_count = p4.next_table(page.p4_index()).unwrap().entry_count();
                if (p3_count - 1) == 0 {
                    let frame = p4[page.p4_index()].pointed_frame().unwrap();
                    p4[page.p4_index()].set_unused();
                    deallocate_frame(frame);
                } else {
                    p4.next_table_mut(page.p4_index()).unwrap().set_entry_count(p3_count - 1);
                }
            } 

        }

        frame
    }

    /// Unmap a page
    pub fn unmap(&mut self, page: Page) -> MapperFlush {
        let frame = self.unmap_inner(&page, false);
        deallocate_frame(frame);
        MapperFlush::new(page)
    }

    /// Unmap a page, return frame without free
    pub fn unmap_return(&mut self, page: Page, keep_parents: bool) -> (MapperFlush, Frame) {
        let frame = self.unmap_inner(&page, keep_parents);
        (MapperFlush::new(page), frame)
    }

}