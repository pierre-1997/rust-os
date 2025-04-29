#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
mod io;
mod allocator;
#[cfg(test)]
mod testing;

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::{config::Mapping, info::FrameBuffer, BootloaderConfig};
use io::{serial::SerialWriter, vga::VGAWriter};

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("PANIC!!! ");
    if let Some(location) = info.location() {
        print!("[{}:{}] ", location.file(), location.line());
    }

    println!("{}", info.message());
    loop {}
}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    // NOTE: We extract the `FrameBuffer` here so that we can still borrow `boot_info` later on
    let mut owned_fb = unsafe {
        let bootloader_api::info::Optional::Some(fb) = &mut boot_info.framebuffer else {
            panic!("Missing framebuffer in boot info.");
        };
        core::ptr::read(fb as *mut FrameBuffer)
    };

    // Initialize VGA and Serial port writing (e.g. text outputs).
    VGAWriter::init(&mut owned_fb);
    SerialWriter::init_serial().expect("Failed to initialize Serial writer.");

    #[cfg(test)]
    {
        test_main();
        // TODO: Exit here
        // io::exit(0);
    }

    println!("HElllozz");
    println!("AGAIN");

    // Initialize allocator.
    allocator::init(boot_info);
    allocator::print_free_segments();

    {
        let mut v: alloc::vec::Vec<usize> = alloc::vec::Vec::with_capacity(10);
        v.push(1);
        v.push(2);
        v.push(3);
        println!("v = {:?}", v);

        let mut v1: alloc::vec::Vec<usize> = alloc::vec::Vec::with_capacity(10);
        v1.push(1);
        v1.push(2);
        v1.push(3);
        println!("v = {:?}", v1);
    }

    loop {}
}

// We force physical memory mapping to our kernel.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
