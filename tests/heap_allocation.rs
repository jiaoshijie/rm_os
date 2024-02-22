#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rm_os::test_prelude::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rm_os::test_prelude::*;
use rm_os::hlt_loop;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    rm_os::init();

    use rm_os::allocator;
    use rm_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {
        memory::init(physical_mem_offset)
    };

    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed!!!");

    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// ----------------------------------------------------------------------------

#[test_case]
fn simple_allocation() {
    use alloc::boxed::Box;
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);

    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    use alloc::vec::Vec;
    let n = 1000;
    let mut v = Vec::new();
    for i in 1..=n {
        v.push(i);
    }
    assert_eq!(v.iter().sum::<u64>(), (n + 1) * n / 2);
}

// This test ensures that the allocator reuses freed memory for subsequent allocations, since it
// will run out of memory otherwise.
#[test_case]
fn many_boxes() {
    use alloc::boxed::Box;
    use rm_os::allocator::HEAP_SIZE;

    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

// NOTE: it will failed when use the bump allocator.
#[test_case]
fn many_boxes_with_a_long_lived_one() {
    use alloc::boxed::Box;
    use rm_os::allocator::HEAP_SIZE;

    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}
