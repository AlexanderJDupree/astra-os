// lib.rs - Astra OS Common Library


#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


pub mod gdt;
pub mod memory;
pub mod serial;
pub mod interrupts;
pub mod vga_buffer;


use core::panic:: PanicInfo;
use ansi_rgb::{ Foreground, red, green };

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

//////////////////////////////
// Data Structures and Types
//////////////////////////////

// Qemu translates IO exit codes with (x << 1) | 1. Can't use 0 or 1 since those
// are default exit codes for Qemu. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] 
pub enum QemuExitCode {
    Success = 0x10,
    Failed  = 0x11,
}


pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where T:Fn(),
{
    fn run(&self) {
        serial_print!("{}. . . . ", core::any::type_name::<T>());
        self();
        serial_println!("{}", "[ ok ]".fg(green()));
    }
}


//////////////////////////////
// API
//////////////////////////////

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() }; 
    x86_64::instructions::interrupts::enable();
}


#[cfg(test)] // Cargo xtest entry point
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}


pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        // See Cargo.toml, iobase=0xf4
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}


pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{}", "[ failed ]\n".fg(red()));
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

