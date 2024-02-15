#![no_std]
#![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(rm_os::test_runner)]
// #![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::prelude::*;

#[no_mangle] // Don't add garbage to the function name, leave it as what it is.
pub extern "C" fn _start() -> ! {
    // This function is the entry point for rm_os, because the linker looks for a function named `_start`
    println!("Hello World{}", '!');

    rm_os::init();
    x86_64::instructions::interrupts::int3();

    print!("It didn't crash, anymore{}", "!!!");

    loop {}
}

// This function will be called when a panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
