#![no_std]
#![feature(abi_x86_interrupt)]
// #![cfg_attr(test, no_main)]
// #![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
// #![reexport_test_harness_main = "test_main"]


pub mod vga_buf;
pub mod serial;
pub mod interrupts;
pub mod gdt;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}

pub mod prelude {
    pub use crate::{println, print};
}

pub mod test_prelude {
    use core::panic::PanicInfo;
    use crate::*;

    // Quit qemu
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u32)]
    pub enum QemuExitCode {
        Success = 0x10,
        Failed = 0x11,
    }

    pub fn exit_qemu(exit_code: QemuExitCode) {
        use x86_64::instructions::port::Port;
        unsafe {
            let mut port = Port::new(0xf4);
            port.write(exit_code as u32);
        }
    }

    pub trait Testable {
        fn run(&self) -> ();
    }

    impl<T: Fn()> Testable for T {
        fn run(&self) -> () {
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("\x1b[36m[ok]\x1b[0m");
        }
    }

    pub fn test_panic_handler(info: &PanicInfo) -> ! {
        serial_println!("\x1b[31m[Failed]\x1b[0m");
        serial_println!("Error: {}", info);
        exit_qemu(QemuExitCode::Failed);
        loop {}
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests!!!", tests.len());
        for test in tests {
            test.run();
        }
        exit_qemu(QemuExitCode::Success);
    }
}
