#![no_std]
#![no_main]

#[macro_use]
mod screen;
mod allocator;

extern crate alloc;

#[cfg(not(test))]
use core::panic::PanicInfo;

use bootloader_api::{config::Mapping, info::FrameBuffer, BootloaderConfig};
use screen::ScreenWriter;

/// This function is called on panic.
#[cfg(not(test))]
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
    let bootloader_api::info::Optional::Some(fb) = &mut boot_info.framebuffer else {
        panic!("No framebuffer given in the boot info.");
    };

    let buffer = unsafe {
        let owned = core::ptr::read(fb as *mut FrameBuffer);

        owned.into_buffer()
    };

    ScreenWriter::init(buffer, fb.info());

    println!("HElllozz");
    println!("AGAIN");

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
