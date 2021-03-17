// interrupts.rs - x86 Interrupt Descriptor Table definition and handlers


use crate::gdt;
use crate::{print, println};

use spin;
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};


////////////////////////////////
// Statics/Constants
////////////////////////////////

/**
 *                      ____________                          ____________
 *  Real Time Clock --> 0            |   Timer -------------> 0            |
 *  ACPI -------------> 1            |   Keyboard-----------> 1            |      _____
 *  Available --------> 2 Secondary  |----------------------> 2 Primary    |     |     |
 *  Available --------> 3 Interrupt  |   Serial Port 2 -----> 3 Interrupt  |---> | CPU |
 *  Mouse ------------> 4 Controller |   Serial Port 1 -----> 4 Controller |     |_____|
 *  Co-Processor -----> 5            |   Parallel Port 2/3 -> 5            |
 *  Primary ATA ------> 6            |   Floppy disk -------> 6            |
 *  Secondary ATA ----> 7____________|   Parallel Port 1----> 7____________|
 * 
 *  Set offsets for the Programmable Interrupt Controllers. By default, the 8259
 *  PIC uses interrupt vectors that are already mapped to CPU exceptions. We 
 *  offset the PIC to use the range 32-47 since these are not in use yet. 
 * 
 */
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);

        idt
    };
}


//////////////////////////////
// Functions
//////////////////////////////

pub fn init_idt() {
    IDT.load();
}


// {:#?} - Pretty print debug info
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT - Err {}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}


//////////////////////////////
// Tests
//////////////////////////////

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
