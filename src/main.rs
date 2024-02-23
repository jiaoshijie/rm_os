#![no_std]
#![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(rm_os::test_runner)]
// #![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rm_os::prelude::*;

entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> ! {
    rm_os::init();

    use rm_os::allocator;
    use rm_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_mem_offset) };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed!!!");

    let x = Box::new(10);
    println!("{:p}", x);

    let mut v = Vec::new();
    for i in 0..250 {
        v.push(i);
    }
    println!("{:p}", v.as_slice());

    println!("It did not crash!!!");
    rm_os::hlt_loop();
}

// This function will be called when a panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rm_os::hlt_loop();
}
