use core::alloc::{GlobalAlloc, Layout, Opaque};
use core::ptr::NonNull;
use spin::Mutex;
use linked_list_allocator::Heap;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 80 * 4096; // 320 KiB


static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub unsafe fn init(offset: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(offset, size));
}

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate_first_fit(layout).ok().map_or(0 as *mut Opaque, |allocation| {
                allocation.as_ptr()
            })
        } else {
            panic!("__rust_allocate: heap not initialized");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(NonNull::new_unchecked(ptr), layout)
        } else {
            panic!("__rust_deallocate: heap not initialized");
        }
    }
    
    fn oom(&self) -> ! {
        panic!("Out of memory");
    }
}

