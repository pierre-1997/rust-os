#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
mod io;
mod allocator;
mod interrupts;
#[cfg(test)]
mod testing;
mod utils;

extern crate alloc;

use core::{cell::OnceCell, panic::PanicInfo};

use bootloader_api::{config::Mapping, info::FrameBuffer, BootloaderConfig};
use io::{serial::SerialWriter, vga::VGAWriter};

struct U64Cell(OnceCell<u64>);
// Safety: We're in single thread for now.
unsafe impl Sync for U64Cell {}

static PHYS_MEM_OFFSET: U64Cell = U64Cell(OnceCell::new());

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("\nPANIC!!! ");
    if let Some(location) = info.location() {
        print!("[{}:{}] ", location.file(), location.line());
    }

    println!("{}", info.message());

    loop {}
    io::exit(1);
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
        loop {}
        io::exit(1);
    }

    // We only work using mapped physical memory.
    let bootloader_api::info::Optional::Some(physical_memory_offset) =
        boot_info.physical_memory_offset
    else {
        panic!("Physical memory is not mapped !!");
    };
    println!("Physical memory offset: {:#X}", physical_memory_offset);

    // Safety: This is the first time we access `PHYS_MEM_OFFSET`.
    let _ = PHYS_MEM_OFFSET.0.set(physical_memory_offset);

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

    // Initialize interrupts
    interrupts::init();

    println!("It did not crash.");

    loop {}
    io::exit(0);
}

// We force physical memory mapping to our kernel.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
