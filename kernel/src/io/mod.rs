use core::arch::asm;

pub mod serial;
pub mod vga;

unsafe fn inb(port: u16) -> u8 {
    let mut ret;

    asm!(
        "in %dx, %al",
        in("dx") port,
        out("al") ret,
        options(att_syntax)
    );

    ret
}

unsafe fn outb(port: u16, value: u8) {
    asm!(
        r#"
        out %al, %dx
        "#,
        in("dx") port,
        in("al") value,
        options(att_syntax)
    );
}

macro_rules! print {
    ($($arg:tt)*) => {
        unsafe {
            use core::fmt::Write as FmtWrite;
            let writer = match (*$crate::io::vga::SCREEN_WRITER.0.get()).as_mut() {
                Some(w) => w,
                None => {
                    panic!("Attempted to use ScreenWriter before calling init.")
                }
            };

            write!(&mut *(writer), $($arg)*).expect("Failed to write to VGA.");

            let mut writer = match (*$crate::io::serial::SERIAL_WRITER.0.get()).as_mut()
                {
                Some(w) => w,
                None => {
                    panic!("Attempted to use SerialWriter before calling init.")
                }
            };

            write!(&mut (writer), $($arg)*).expect("Failed to write in serial.");
        }
    }
}

macro_rules! println {
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    }
}
