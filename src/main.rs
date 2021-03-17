//  main.rs - Astra OS entry point


#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(astra_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


use astra_os::println;
use core::panic::PanicInfo;


#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello {}", "astra-os");

    astra_os::init();

    #[cfg(test)]
    test_main();

    println!("Phew, I didn't crash. . .");
    astra_os::hlt_loop();
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    astra_os::hlt_loop();
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    astra_os::test_panic_handler(info)
}

