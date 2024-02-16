// Task State Segment(TSS):
//
//   Privilege Stack Table(PST) [3 pointers]
//
//   Interrupt Stack Table(IST) [7 pointers]
//
//   I/O Map Base Address (NOTE: maybe, I/O port permissions bitmap)

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use core::ptr::addr_of;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;  // TODO: Why the type is u16?

// TODO: Because the kernal has not implemented memory management yet, so for
// now, we just simple use the static memory as the IST for double fault stack.
// We'll change it later.
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Rust can do this, Cool!!!
            const STACK_SIZE: usize = 4096 * 5;  // 20KB
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

// GPT(Global Descriptor Table) is for memory segmentation(which is used before paging),
// in 64-bit mode, the GPT is only used for two things:
// 1. Switching between kernel space and user space
// 2. loading a TSS structure

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors {code_selector, tss_selector})
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
