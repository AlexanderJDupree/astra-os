// stack_overflow.rs - Test exception for stack overflows

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use ansi_rgb::{Foreground, red, green};
use astra_os::{serial_print, serial_println, exit_qemu, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};


lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(astra_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow:. . . . ");

    astra_os::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("{}", "[ Execution continued after stack overflow ]".fg(red()));
}


extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame, 
    _error_code: u64
) -> ! {
    serial_println!("{}", "[ ok ]".fg(green()));
    exit_qemu(QemuExitCode::Success);
    loop {}
}


pub fn init_test_idt() {
    TEST_IDT.load();
}


#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    // Prevent tail recursion optimizations
    volatile::Volatile::new(0).read();
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    astra_os::test_panic_handler(info);
}
