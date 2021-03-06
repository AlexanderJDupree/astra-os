//  main.rs - Astra OS entry point


#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(astra_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


use astra_os::println;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};


entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use astra_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    astra_os::init();

    println!("Hello {}", "astra-os");

    let phy_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phy_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

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

