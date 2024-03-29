#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rm_os::test_prelude::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::serial_println;
use rm_os::test_prelude::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
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

// Overwrite the panic function, let it exit_qemu success when panic occurs.

#[test_case]
fn test_should_panic() {
    assert_eq!(0, 1);
    exit_qemu(QemuExitCode::Failed);
}
