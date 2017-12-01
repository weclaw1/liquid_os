mod entry;
mod table;
use ::PAGE_SIZE;
use ::Frame;

use self::table::{Table, Level4};
use core::ptr::Unique;

pub use self::entry::*;
use ::FrameAllocator;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

const ENTRY_COUNT: usize = 512;

pub struct Page {
   number: usize,
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virtual_address: VirtualAddress)
    -> Option<PhysicalAddress>
    {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
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
        let (new_entry_p3, new_entry_p2) =
        {
            let (p3, _) = self.p4_mut().next_table_create(page.p4_index(), allocator);
        
            let (p2, new_entry_p3) = p3.next_table_create(page.p3_index(), allocator);

            let (p1, new_entry_p2) = p2.next_table_create(page.p2_index(), allocator);

            assert!(p1[page.p1_index()].is_unused());
            p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
            (new_entry_p3, new_entry_p2)
        };

        if new_entry_p3 {
            let p4 = self.p4_mut();
            let current_count = p4[page.p4_index()].inner_count();
            p4[page.p4_index()].set_inner_count(current_count+1);
        }

        if new_entry_p2 {
            let p3 = self.p4_mut().next_table_mut(page.p4_index()).unwrap();
            let current_count = p3[page.p3_index()].inner_count();
            p3[page.p3_index()].set_inner_count(current_count+1);
        }

        let p2 = self.p4_mut().next_table_mut(page.p4_index())
                              .and_then(|p3| p3.next_table_mut(page.p3_index())).unwrap();
        let current_count = p2[page.p2_index()].inner_count();
        p2[page.p2_index()].set_inner_count(current_count+1);
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

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_address()).is_some());

        {
            let p1 = self.p4_mut()
                         .next_table_mut(page.p4_index())
                         .and_then(|p3| p3.next_table_mut(page.p3_index()))
                         .and_then(|p2| p2.next_table_mut(page.p2_index()))
                         .expect("mapping code does not support huge pages");
            let frame = p1[page.p1_index()].pointed_frame().unwrap();
            p1[page.p1_index()].set_unused();
            allocator.deallocate_frame(frame);
        }

        let p2_freed = {
            let p2 = self.p4_mut()
                         .next_table_mut(page.p4_index())
                         .and_then(|p3| p3.next_table_mut(page.p3_index()))
                         .unwrap();
            let current_count = p2[page.p2_index()].inner_count();
            if (current_count - 1) == 0 {
                let frame = p2[page.p2_index()].pointed_frame().unwrap();
                p2[page.p2_index()].set_unused();
                allocator.deallocate_frame(frame);
                true
            } else {
                p2[page.p2_index()].set_inner_count(current_count - 1);
                false
            }
        };

        if p2_freed {
            let p3_freed = {
                let p3 = self.p4_mut()
                             .next_table_mut(page.p4_index())
                             .unwrap();
                let current_count = p3[page.p3_index()].inner_count();
                if (current_count - 1) == 0 {
                    let frame = p3[page.p3_index()].pointed_frame().unwrap();
                    p3[page.p3_index()].set_unused();
                    allocator.deallocate_frame(frame);
                    true
                } else {
                    p3[page.p3_index()].set_inner_count(current_count - 1);
                    false
                }
            };

            if p3_freed {
                let p4 = self.p4_mut();

                let current_count = p4[page.p4_index()].inner_count();
                if (current_count - 1) == 0 {
                    let frame = p4[page.p4_index()].pointed_frame().unwrap();
                    p4[page.p4_index()].set_unused();
                    allocator.deallocate_frame(frame);
                } else {
                    p4[page.p4_index()].set_inner_count(current_count - 1);
                }
            } 

        }
    }


}

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
                address >= 0xffff_8000_0000_0000,
                "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}
