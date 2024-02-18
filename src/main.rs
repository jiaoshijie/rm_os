#![no_std]
#![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(rm_os::test_runner)]
// #![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::prelude::*;

#[no_mangle] // Don't add garbage to the function name, leave it as what it is.
             // NOTE: This function is the entry point for rm_os, because the linker looks for a function named `_start`
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", '!');

    rm_os::init();

    // NOTE: issue a `int3` interrupt
    // x86_64::instructions::interrupts::int3();

    use x86_64::registers::control::Cr3;
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table physical address: {:?}", level_4_page_table.start_address());

    // NOTE: Cheers
    println!("Cheers, It didn't crash!!!");
    rm_os::hlt_loop();
}

// This function will be called when a panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rm_os::hlt_loop();
}
