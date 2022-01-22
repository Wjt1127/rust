use crate::alloc::{GlobalAlloc, Layout, System};
use crate::ptr;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        twizzler_abi::alloc::global_alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let addr = twizzler_abi::alloc::global_alloc(layout);

        if !addr.is_null() {
            ptr::write_bytes(addr, 0x00, layout.size());
        }

        addr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        twizzler_abi::alloc::global_free(ptr, layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        twizzler_abi::alloc::global_realloc(ptr, layout, new_size)
    }
}
