#![no_std]
#![feature(abi_x86_interrupt)]
// #![cfg_attr(test, no_main)]
// #![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]
// #![reexport_test_harness_main = "test_main"]

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buf;

pub fn init() {
    gdt::init(); // NOTE: For now it for switch stack when double fault occurs.
    interrupts::init_idt(); // Config the interrupt handler function
    unsafe { interrupts::PICS.lock().initialize() }; // Config external interrupt numbers.
    x86_64::instructions::interrupts::enable(); // Enable external interrupts.
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub mod prelude {
    pub use crate::{print, println};
}

pub mod test_prelude {
    use crate::*;
    use core::panic::PanicInfo;

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
        hlt_loop();
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests!!!", tests.len());
        for test in tests {
            test.run();
        }
        exit_qemu(QemuExitCode::Success);
    }
}
