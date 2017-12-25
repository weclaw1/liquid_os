use super::Page;
use super::{ActivePageTable, VirtualAddress};
use super::table::{Table, Level1};
use memory::Frame;
use super::entry::EntryFlags;

pub struct TemporaryPage {
    page: Page,
}

impl TemporaryPage {
    pub fn new(page: Page) -> TemporaryPage {
        TemporaryPage {
            page: page,
        }
    }

    pub fn start_address (&self) -> VirtualAddress {
        self.page.start_address()
    }

    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        assert!(active_table.translate_page(self.page).is_none(), "temporary page is already mapped");
        let result = active_table.map_to(self.page, frame, EntryFlags::WRITABLE);
        result.flush(active_table);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        let (result, _frame) = active_table.unmap_return(self.page, true);
        result.flush(active_table);
    }

    /// Maps the temporary page to the given page table frame in the active
    /// table. Returns a reference to the now mapped table.
    pub fn map_table_frame(&mut self, frame: Frame, active_table: &mut ActivePageTable) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }

}