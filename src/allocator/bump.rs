use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/*
 * bump allocator is seldom used as the global allocator
 *
 * Advantage:
 *   1. Fast, compared to other allocators
 * Drawback:
 *   1. It can only reuse deallocated memory after all allocations have been freed.
*/

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

// NOTE: Creating a separate init function instead of initializing in the new function is in
// order to keep the same interface provided by the linked_list_allocator crate
impl BumpAllocator {
    /// Create a new empty bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let addr_start = align_up(bump.next, layout.align());
        let Some(addr_end) = addr_start.checked_add(layout.size()) else {
            return ptr::null_mut();
        };

        if addr_end > bump.heap_end {
            ptr::null_mut() // out of heap memory
        } else {
            bump.next = addr_end;
            bump.allocations += 1;
            addr_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        // TODO: Maybe need to check double free.
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
