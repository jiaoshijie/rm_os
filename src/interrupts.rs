use crate::prelude::*;
use crate::{gdt, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[ExternalInterruptIndex::Timer.into()].set_handler_fn(timer_interrupt_handler);
        idt[ExternalInterruptIndex::Keyboard.into()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// NOTE: it's a diverging function
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE_FAULT(error code: {}): \n{:#?}",
        err_code, stack_frame
    );
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(ExternalInterruptIndex::Timer.into());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // TODO: What is the difference between port and serial port?
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    // add scancode to a global buffer
    crate::task::keyboard::add_scancode(scancode);

    // NOTE: this code below should be handled by other tasks.
    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    //     if let Some(key) = keyboard.process_keyevent(key_event) {
    //         match key {
    //             DecodedKey::Unicode(ch) => print!("{ch}"),
    //             DecodedKey::RawKey(key) => print!("{key:?}"),
    //         }
    //     }
    // }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(ExternalInterruptIndex::Keyboard.into());
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use crate::hlt_loop;
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop(); // TODO: Why is this fucntion used in here?
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ExternalInterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // PIC_1_OFFSET + 1
}

impl From<ExternalInterruptIndex> for u8 {
    fn from(value: ExternalInterruptIndex) -> Self {
        value as u8
    }
}

impl From<ExternalInterruptIndex> for usize {
    fn from(value: ExternalInterruptIndex) -> Self {
        value as usize
    }
}
