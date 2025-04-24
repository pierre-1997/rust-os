#![no_std]
#![no_main]

#[macro_use]
mod screen;

#[cfg(not(test))]
use core::panic::PanicInfo;

use bootloader_api::info::FrameBuffer;
use screen::ScreenWriter;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("PANIC!!! ");
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

    loop {}
}

/* This is how we can ship custom configurations to our kernel.
*
    pub static BOOTLOADER_CONFIG: BootloaderConfig = {
        let mut config = BootloaderConfig::new_default();
        config.mappings.physical_memory = Some(Mapping::Dynamic);
        config
    };
*/

bootloader_api::entry_point!(kernel_main);
