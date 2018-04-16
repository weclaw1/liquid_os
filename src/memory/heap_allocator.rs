use core::alloc::{GlobalAlloc, Layout};
use core::alloc::Opaque;
use spin::Mutex;
use linked_list_allocator::{Heap, LockedHeap, ALLOCATOR};

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 80 * 4096; // 320 KiB


static HEAP: Mutex<Option<&mut Heap>> = Mutex::new(None);

pub unsafe fn init(offset: usize, size: usize) {
    ALLOCATOR.init(offset, size);
    *HEAP.lock() = Some(&mut ALLOCATOR);
}

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout)
        } else {
            panic!("__rust_allocate: heap not initialized");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(ptr, layout)
        } else {
            panic!("__rust_deallocate: heap not initialized");
        }
    }
    
    fn oom(&self) -> ! {
        panic!("Out of memory");
    }
}

