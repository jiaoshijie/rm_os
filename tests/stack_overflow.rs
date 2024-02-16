#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
// NOTE: Since we have configured `harness` in Cargo.toml, these lines below are not useful anymore.
// #![feature(custom_test_frameworks)]
// #![test_runner(rm_os::test_prelude::test_runner)]
// #![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rm_os::test_prelude::*;
use rm_os::serial_println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rm_os::gdt::init();
    init_test_idt();

    stack_overflow();  // NOTE: make stack overflow

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

// ----------------------------------------------------------------------------

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT_TEST: InterruptDescriptorTable = {
        let mut idt_test = InterruptDescriptorTable::new();
        unsafe {
            idt_test.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rm_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt_test
    };
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    // serial_println!("EXCEPTION: DOUBLE_FAULT(error code: {}): \n{:#?}", err_code, stack_frame);
    serial_println!("stack_overflow::stack_overflow...\x1b[36m[ok]\x1b[0m");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

pub fn init_test_idt() {
    IDT_TEST.load();
}
