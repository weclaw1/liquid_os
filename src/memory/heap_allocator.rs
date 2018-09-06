use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use spin::Mutex;
use slab_allocator::Heap;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 80 * 4096; // 320 KiB


static HEAP: Mutex<Option<Heap>> = Mutex::new(None);

pub unsafe fn init(offset: usize, size: usize) {
    *HEAP.lock() = Some(Heap::new(offset, size));
}

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.allocate(layout).ok().map_or(0 as *mut u8, |allocation| {
                allocation.as_ptr()
            })
        } else {
            panic!("__rust_allocate: heap not initialized");
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ref mut heap) = *HEAP.lock() {
            heap.deallocate(NonNull::new_unchecked(ptr), layout)
        } else {
            panic!("__rust_deallocate: heap not initialized");
        }
    }

}

#[cfg(not(test))]
#[alloc_error_handler]
#[no_mangle]
pub extern "C" fn oom(_: ::core::alloc::Layout) -> ! {
    panic!("kernel memory allocation failed");
}
