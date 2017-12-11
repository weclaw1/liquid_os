use super::{VirtualAddress, PhysicalAddress, Page, ENTRY_COUNT};
use super::entry::*;
use super::table::{self, Table, Level4, Level1};
use ::{PAGE_SIZE, Frame, FrameAllocator};
use core::ptr::Unique;
use extern_x86_64;
use extern_x86_64::instructions::tlb;

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

    pub fn translate(&self, virtual_address: VirtualAddress)
    -> Option<PhysicalAddress>
    {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() *
                                    ENTRY_COUNT + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index()
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
        .and_then(|p2| p2.next_table(page.p2_index()))
        .and_then(|p1| p1[page.p1_index()].pointed_frame())
        .or_else(huge_page)
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {   
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());

        let current_count = p1.entry_count();
        p1.set_entry_count(current_count + 1);

        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn unmap_inner<A>(&mut self, page: &Page, allocator: &mut A) -> Frame
        where A: FrameAllocator
    {
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
                allocator.deallocate_frame(frame);
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
                    allocator.deallocate_frame(frame);
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
                    allocator.deallocate_frame(frame);
                } else {
                    p4.next_table_mut(page.p4_index()).unwrap().set_entry_count(p3_count - 1);
                }
            } 

        }

        frame
    }


    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator 
    {
        let frame = self.unmap_inner(&page, allocator);
        allocator.deallocate_frame(frame);
        tlb::flush(extern_x86_64::VirtualAddress(page.start_address()));
    }

    /// Unmap a page, return frame without free
    pub fn unmap_return<A>(&mut self, page: Page, allocator: &mut A) -> Frame 
    where A: FrameAllocator 
    {
        let frame = self.unmap_inner(&page, allocator);
        frame
    }

}