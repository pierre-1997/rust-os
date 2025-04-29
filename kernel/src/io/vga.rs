//! For now, we use the `noto_sans_mono_bitmap` crate in order to load only that specific font.
//! Later on, we could figure out how to ship a font directly during compilation and then load it
//! from within our kernel.
//!
//! NOTE:
//! - `0xff` = White
//! - `0x00` = Black
//!
//! TODO:
//! - Font color (background & foreground) support?!
//! - We only support `3 bytes per pixel` formats ?
//! - Investigate: For now, `'�'` as a backup char seems to crash stuff.

use core::{cell::UnsafeCell, fmt::Write};

use bootloader_api::info::FrameBuffer;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

const UNKNOWN_CHAR: char = ' '; // '�';
const BG_COLOR: u8 = 0x00; // Black

const HORIZONTAL_BORDER_PADDING: usize = 30;
const VERTICAL_BORDER_PADDING: usize = 30;

const CHAR_SPACING: usize = 0;
const CHAR_HEIGHT: usize = RasterHeight::Size16.val();
const CHAR_WIDTH: usize = get_raster_width(FontWeight::Regular, RasterHeight::Size16);
const LINE_SPACING: usize = 2;

pub struct VGAWriter {
    /// TODO: Put this behind a `Mutex` to allow multiple writers?
    buffer: &'static mut [u8],

    info: bootloader_api::info::FrameBufferInfo,

    cur_x: usize,
    cur_y: usize,

    cur_font_weight: FontWeight,
    cur_font_height: RasterHeight,
}
/// NOTE: We use `UnsafeCell` to achieve interior mutability here.
pub struct VGAWriterHolder(pub UnsafeCell<Option<VGAWriter>>);

unsafe impl Sync for VGAWriterHolder {}

pub static SCREEN_WRITER: VGAWriterHolder = VGAWriterHolder(UnsafeCell::new(None));

impl VGAWriter {
    /// This function initializes `SCREEN_WRITER` given a frame buffer and its relative
    /// information.
    /// It *has to be called* before being able to call the logging macros.
    ///
    ///
    /// NOTE: For now we'll use the build-defaults font sizes (weights and height). If we want to
    /// support more, we just need to change the compile features of `noto_sans_mono_bitmap`.
    pub fn init(fb: &mut FrameBuffer) {
        let info = fb.info();

        // FIXME: For now we force the 3 bytes per pixel formats (e.g. either RGB or BGR).
        assert_eq!(info.bytes_per_pixel, 3);

        let buffer = unsafe {
            let owned = core::ptr::read(fb as *mut FrameBuffer);

            owned.into_buffer()
        };

        let mut writer = Self {
            buffer,
            info,
            cur_x: HORIZONTAL_BORDER_PADDING,
            cur_y: VERTICAL_BORDER_PADDING,
            cur_font_weight: FontWeight::Regular,
            cur_font_height: RasterHeight::Size16,
        };

        // Clear the whole screen.
        writer.clear();

        unsafe {
            SCREEN_WRITER.0.get().write(Some(writer));
        }
    }

    /// Clears the screen and fill it with `BG_COLOR`.
    pub fn clear(&mut self) {
        self.cur_x = HORIZONTAL_BORDER_PADDING;
        self.cur_y = VERTICAL_BORDER_PADDING;

        // Fill with Black.
        self.buffer.fill(BG_COLOR)
    }

    /// Write a single character on the screen at the current position.
    pub fn print_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                // If the char will go over the right border, do a newline
                let new_x = self.cur_x + CHAR_WIDTH;
                if new_x > self.info.width - HORIZONTAL_BORDER_PADDING {
                    self.newline();
                }
                // If the char will go over the bottom border, clear the screen.
                // TODO: Implement screen scrolling ?
                let new_y = self.cur_y + CHAR_HEIGHT;
                if new_y > self.info.height - VERTICAL_BORDER_PADDING {
                    self.clear();
                }

                self.write_rendered_char(self.get_rendered_char(c));
            }
        }
    }

    /// Converts a character to its rendered bitmap.
    fn get_rendered_char(&self, c: char) -> RasterizedChar {
        get_raster(c, self.cur_font_weight, self.cur_font_height).unwrap_or(self.backup_char())
    }

    /// Writes a whole character on the screen.
    fn write_rendered_char(&mut self, char_pixels: RasterizedChar) {
        for (yi, row) in char_pixels.raster().iter().enumerate() {
            for (xi, pixel) in row.iter().enumerate() {
                self.write_pixel(self.cur_x + xi, self.cur_y + yi, *pixel);
            }
        }

        // Update the cursor's horizontal position.
        self.cur_x += char_pixels.width() + CHAR_SPACING;
    }

    /// Writes a single pixel on the screen.
    ///
    /// NOTE: `intensity` is basically a grayscale for now.
    pub fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let idx = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        // NOTE: This could be behind a `hardened` feature.
        assert!(idx < self.info.byte_len);

        // For now, we manually write the three RGB values. Will fix this when adding color
        // support.
        self.buffer[idx] = intensity;
        self.buffer[idx + 1] = intensity;
        self.buffer[idx + 2] = intensity;
    }

    /// Goes to the beginning of the next line.
    fn newline(&mut self) {
        self.cur_y += CHAR_HEIGHT + LINE_SPACING;
        self.carriage_return();
    }

    /// Returns to the beginning of the current line.
    fn carriage_return(&mut self) {
        self.cur_x = HORIZONTAL_BORDER_PADDING;
    }

    /// Gets the default char `UNKNOWN_CHAR` ready to be rendered.
    ///
    /// TODO: Maybe this should be generated only once ever using a `static` ?
    ///
    /// NOTE: This panics if unable to generate an `UNKNOWN_CHAR` with the current font weight and
    /// height.
    fn backup_char(&self) -> RasterizedChar {
        get_raster(UNKNOWN_CHAR, self.cur_font_weight, self.cur_font_height)
            .expect("Failed to get raster of backup char")
    }
}

/// So that we can use the nifty `write!()` macro.
impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.print_char(c);
        }

        Ok(())
    }
}
