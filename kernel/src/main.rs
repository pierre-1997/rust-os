#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let bootloader_api::info::Optional::Some(fb) = &mut boot_info.framebuffer else {
        panic!("No framebuffer given in the boot info.");
    };

    let buffer = fb.buffer_mut();

    for (i, pixel) in buffer.iter_mut().enumerate() {
        match i % 3 {
            0 => *pixel = 0x00,
            1 => *pixel = 0x00,
            2 => *pixel = 0xff,
            _ => unreachable!(),
        }
    }

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
