#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rm_os::test_prelude::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::test_prelude::*;
use rm_os::serial_println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rm_os::interrupts::init_idt();
    test_main();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("\x1b[36m[ok]\x1b[0m");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

// ----------------------------------------------------------------------------

#[test_case]
fn test_int3() {
    x86_64::instructions::interrupts::int3();
}
