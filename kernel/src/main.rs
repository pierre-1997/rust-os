#![no_std]
#![no_main]

mod screen;

use core::fmt::Write;
#[cfg(not(test))]
use core::panic::PanicInfo;

use bootloader_api::info::FrameBuffer;
use screen::ScreenWriter;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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

    let mut screen_writer = ScreenWriter::from_bootinfo(buffer, fb.info());

    for _ in 0..20 {
        screen_writer.print_char('X');
    }
    screen_writer.print_char('�');

    writeln!(screen_writer).expect("ss");

    writeln!(screen_writer, "Hellooooo").expect("WRdf");

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
