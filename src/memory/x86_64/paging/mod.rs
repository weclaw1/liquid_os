mod entry;
mod table;
mod temporary_page;
mod mapper;

use memory::Frame;
use memory::allocate_frame;

pub use self::entry::EntryFlags;
use multiboot2::BootInformation;

use x86_64;
use x86_64::instructions::tlb;
use x86_64::registers::control_regs;

use self::mapper::Mapper;
use core::ops::{Deref, DerefMut};

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub const PAGE_SIZE: usize = 4096;
const ENTRY_COUNT: usize = 512;

use self::temporary_page::TemporaryPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
   number: usize,
}

impl Page {
    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }

    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address < 0x0000_8000_0000_0000 ||
                address >= 0xffff_8000_0000_0000,
                "invalid address: 0x{:x}", address);
        Page { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
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

pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temporary_page: &mut temporary_page::TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper)
    {
        {
            let backup = Frame::containing_address(control_regs::cr3().0 as usize);

            // map temporary_page to current p4 table
            let p4_table = temporary_page.map_table_frame(backup.clone(), self);

            // overwrite recursive mapping
            self.p4_mut()[511].set(table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.flush_all();

            // execute f in the new context
            f(self);

            // restore recursive mapping to original p4 table
            p4_table[511].set(backup, EntryFlags::PRESENT | EntryFlags::WRITABLE);

            self.flush_all();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(
                control_regs::cr3().0 as usize
            ),
        };
        unsafe {
            control_regs::cr3_write(x86_64::PhysicalAddress(new_table.p4_frame.start_address() as u64));
        }
        old_table
    }

    pub fn flush(&mut self, page: Page) {
        tlb::flush(x86_64::VirtualAddress(page.start_address()));
    }

    pub fn flush_all(&mut self) {
        tlb::flush_all();
    }

}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame,
               active_table: &mut ActivePageTable,
               temporary_page: &mut TemporaryPage)
               -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            // now we are able to zero the table
            table.zero();
            // set up recursive mapping for the table
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }

}

pub fn remap_the_kernel(boot_info: &BootInformation) -> ActivePageTable {
    let mut temporary_page = TemporaryPage::new(Page { number: 0xcafebabe });

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocate_frame().expect("no more frames");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        for section in elf_sections_tag.sections() { 
            if !section.is_allocated() {
                // section is not loaded to memory
                continue;
            }
            assert!(section.start_address() % PAGE_SIZE == 0, "sections need to be page aligned");

            println!("mapping section at addr: {:#x}, size: {:#x}", section.addr, section.size);

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                let result = mapper.identity_map(frame, flags);
                // The flush can be ignored as this is not the active table. See later active_table.switch
                unsafe {result.ignore();}
            }
        }
        // identity map the VGA text buffer
        let vga_buffer_frame = Frame::containing_address(0xb8000);
        let result = mapper.identity_map(vga_buffer_frame, EntryFlags::WRITABLE);
        // The flush can be ignored as this is not the active table. See later active_table.switch
        unsafe {result.ignore();}

        // identity map the multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            let result = mapper.identity_map(frame, EntryFlags::PRESENT);
            // The flush can be ignored as this is not the active table. See later active_table.switch
            unsafe {result.ignore();}
        }
    });

    let old_table = active_table.switch(new_table);
    println!("NEW TABLE!!!");

    // turn the old p4 page into a guard page
    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    let result = active_table.unmap(old_p4_page);
    result.flush(&mut active_table);
    println!("guard page at {:#x}", old_p4_page.start_address());
    active_table
}
