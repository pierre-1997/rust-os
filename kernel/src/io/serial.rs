use core::{cell::UnsafeCell, fmt::Write};

use super::{inb, outb};

pub struct SerialWriter;
pub struct SerialWriterHolder(pub UnsafeCell<Option<SerialWriter>>);

unsafe impl Sync for SerialWriterHolder {}

pub static SERIAL_WRITER: SerialWriterHolder = SerialWriterHolder(UnsafeCell::new(None));
/// COM1 serial port within Qemu.
const PORT: u16 = 0x3f8;

/// Checks if there is already something being transmitted.
unsafe fn is_transmit_empty() -> bool {
    (inb(PORT + 5) & 0x20) != 0
}

/// Writes a single byte on the serial port.
unsafe fn write_byte(b: u8) {
    while !is_transmit_empty() {}

    outb(PORT, b);
}

#[derive(Debug)]
pub enum SerialError {
    InitFailed,
}

impl SerialWriter {
    pub fn init_serial() -> Result<(), SerialError> {
        unsafe {
            outb(PORT + 1, 0x00); // Disable all interrupts
            outb(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
            outb(PORT, 0x03); // Set divisor to 3 (lo byte) 38400 baud
            outb(PORT + 1, 0x00); //                  (hi byte)
            outb(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
            outb(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
            outb(PORT + 4, 0x1E); // Set in loopback mode, test the serial chip
            outb(PORT, 0xAE); // Test serial chip (send byte 0xAE and check if serial returns same byte)

            // Check if serial is faulty (i.e: not same byte as sent)
            if inb(PORT) != 0xAE {
                return Err(SerialError::InitFailed);
            }

            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            outb(PORT + 4, 0x0F);

            SERIAL_WRITER.0.get().write(Some(SerialWriter {}));
        }

        Ok(())
    }
}

/// So that we can use the nifty `write!()` macro.
impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            for b in s.bytes() {
                write_byte(b);
            }
        }

        Ok(())
    }
}
