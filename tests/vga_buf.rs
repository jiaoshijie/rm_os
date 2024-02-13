#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rm_os::test_prelude::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::test_prelude::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// ----------------------------------------------------------------------------

use rm_os::prelude::*;
use rm_os::vga_buf::*;

#[test_case]
fn style1() {
    assert_eq!(0b01011100u8, Style::new(Color::Red, Color::Magenta, true, false).code());
}

#[test_case]
fn print1() {
    let printable = "abcdefghijklmnopqrstuvwxyz";
    for ch in printable.chars() {
        println!("{ch}");
    }

    let mut bytes = printable.bytes();
    bytes.next();
    bytes.next();
    let lock = WRITER.lock();
    for i in 0..BUFFER_HEIGHT {
        if let Some(byte) = bytes.next() {
            assert_eq!(byte, lock.vga_buf.chars[i][0].read().ch);
        } else {
            assert_eq!(' ' as u8, lock.vga_buf.chars[i][0].read().ch);
        }
    }
}
