use memory::Frame;
use multiboot2::ElfSection;
use multiboot2::{ELF_SECTION_ALLOCATED, ELF_SECTION_WRITABLE, ELF_SECTION_EXECUTABLE};

const ADDRESS_MASK: usize = 0x000f_ffff_ffff_f000;
const COUNTER_MASK: u64 = 0x3ff00000_00000000;

pub struct Entry(u64);

impl Entry {
    /// Zero entry
    pub fn set_zero(&mut self) {
        self.0 = 0;
    }

    /// Is the entry unused?
    pub fn is_unused(&self) -> bool {
        self.0 == self.0 & COUNTER_MASK
    }

    /// Make the entry unused
    pub fn set_unused(&mut self) {
        self.0 = self.0 & COUNTER_MASK;
    }

    /// Get the current entry flags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Get the associated frame, if available
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(self.0 as usize & ADDRESS_MASK))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        debug_assert!(frame.start_address() & !ADDRESS_MASK == 0);
        self.0 = (frame.start_address() as u64) | flags.bits() | (self.0 & COUNTER_MASK);
    }

    /// Get bits 52-61 in entry, used as counter for page table
    pub fn counter_bits(&self) -> u64 {
        (self.0 & COUNTER_MASK) >> 52
    }

    /// Set bits 52-61 in entry, used as counter for page table    
    pub fn set_counter_bits(&mut self, count: u64) {
        self.0 = (self.0 & !COUNTER_MASK) | (count << 52);
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}

impl EntryFlags {
    pub fn from_elf_section_flags(section: &ElfSection) -> EntryFlags {
        let mut flags = EntryFlags::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            // section is loaded to memory
            flags = flags | EntryFlags::PRESENT;
        }
        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags = flags | EntryFlags::WRITABLE;
        }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            flags = flags | EntryFlags::NO_EXECUTE;
        }

        flags
    }
}