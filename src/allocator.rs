use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
// use linked_list_allocator::LockedHeap;

pub mod bump;
pub mod linked_list;

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    // NOTE: straightforward
    // let remainder = addr % align;
    // if remainder == 0 {
    //     addr
    // } else {
    //     addr + align - remainder
    // }

    // NOTE: much faster
    // Require: `align` must be power of two
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
// static ALLOCATOR: DummyAlloc = DummyAlloc;  // NOTE: only return null ptr
// static ALLOCATOR: LockedHeap = LockedHeap::empty();  // NOTE: external crate
// static ALLOCATOR: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());
static ALLOCATOR: Locked<linked_list::LinkedListAllocator> =
    Locked::new(linked_list::LinkedListAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct DummyAlloc;

unsafe impl GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        panic!("daalloc should be never called!!!");
    }
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}
