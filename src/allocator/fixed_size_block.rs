use alloc::alloc::{Layout, GlobalAlloc};
use core::{ptr::{self, NonNull}, mem};
use super::Locked;

/// The block sizes to use
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (that must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

fn list_index(layout: &Layout) -> Option<usize> {
    // TODO: this line maybe is not needed, because rust compiler seems will increase the size
    // equal to align if the size is lower than align
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}


/*
 * fixed size block allocator is better than linked list allocator as a OS kernal
 *
 * Advantage:
 *   1. Better performance than linked list allocator
 * Drawback:
 *   1. Waste some memory, if using powers of 2 as block sizes, it wastes up to half of the memory.
*/

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,  // For now, this is a external crate
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        // TODO: Why cna I create a EMPTY constant value to initialize the array.
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        if let Ok(ptr) = self.fallback_allocator.allocate_first_fit(layout) {
            ptr.as_ptr()
        } else {
            ptr::null_mut()
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        if let Some(index) = list_index(&layout) {
            if let Some(node) = allocator.list_heads[index].take() {
                allocator.list_heads[index] = node.next.take();
                node as *mut ListNode as *mut u8
            } else {
                let block_size = BLOCK_SIZES[index];
                let block_align = block_size;
                let layout = Layout::from_size_align(block_size, block_align).unwrap();
                allocator.fallback_alloc(layout)
            }
        } else {
            allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };

            assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);

            let new_node_ptr = ptr as *mut ListNode;
            new_node_ptr.write(new_node);
            allocator.list_heads[index] = Some(&mut *new_node_ptr);
        } else {
            let ptr = NonNull::new(ptr).unwrap();
            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}
