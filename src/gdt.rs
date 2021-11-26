// gdt.rs - Global Descriptor Table

use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

//////////////////////////////
// Statics/Constants
//////////////////////////////

// Double Fault exception requires its own stack. If we have a stack overflow
// That causes a page fault, the CPU will try to push the interrupt stack frame
// onto the stack that has already overflowed, causing a second page fault which
// triggers the double fault. The CPU will then try to push the exception stack
// frame again onto the bad stack before calling the double fault handler,
// which will cause a triple fault and system reset.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 2Kb

            // Initialize stack to zero
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // TODO add guard page

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end   = stack_start + STACK_SIZE;
            // Stack grows 'downward' so we initialize the stack to to the end
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

////////////////////////////////
// Structs, Types, Traits, Impl
////////////////////////////////

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

//////////////////////////////
// API
//////////////////////////////

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // Reload the code segment register
        set_cs(GDT.1.code_selector);
        // Load the TaskStateSegment
        load_tss(GDT.1.tss_selector);
    }
}
