#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle] // Don't add garbage to the function name, leave it as what it is.
pub extern "C" fn _start() -> ! {
    // This function is the entry point for rm_os, because the linker looks for a function named `_start`
    println!("hello world");
    loop {}
}

// This function will be called when a panic occurs.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
