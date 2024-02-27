#![no_std]
#![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(rm_os::test_runner)]
// #![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rm_os::prelude::*;
use rm_os::allocator;
use rm_os::memory::{self, BootInfoFrameAllocator};
use x86_64::VirtAddr;


entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> ! {
    // Setup basic things
    rm_os::init();

    // Initialize the heap for kernal
    let physical_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed!!!");

    use rm_os::task::{Task, executor::Executor, keyboard::print_keypresses};

    let mut executor = Executor::new();
    executor.spawn(Task::new(print_keypresses()));
    executor.spawn(Task::new(example_task_1()));
    executor.spawn(Task::new(example_task_2()));
    executor.run();
}

async fn async_number_42() -> usize {
    42
}

async fn async_number_43() -> usize {
    43
}

async fn example_task_1() {
    let number = async_number_42().await;
    println!("{}", number);
}

async fn example_task_2() {
    let number = async_number_43().await;
    println!("{}", number);
}

// This function will be called when a panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rm_os::hlt_loop();
}
