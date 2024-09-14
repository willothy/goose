use spin::{lazy::Lazy, Mutex};
use x86_64::instructions::interrupts::without_interrupts;

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

pub static mut WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| {
    Mutex::new(Writer {
        column_pos: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer_ptr: unsafe { &mut *(0xb8000 as *mut Buffer) },
    })
});

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer_ptr: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.newline();
            }
            b'\r' => {
                self.column_pos = 0;
            }
            byte => {
                if self.column_pos >= VGA_WIDTH {
                    self.newline();
                }
                let row = VGA_HEIGHT - 1;
                let col = self.column_pos;
                let color_code = self.color_code;
                self.buffer_ptr.chars[row][col] = ScreenChar {
                    ascii_char: byte,
                    color_code,
                };
                self.column_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        s.bytes().for_each(|byte| match byte {
            0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte),
            _ => self.write_byte(0xfe),
        });
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        self.buffer_ptr.chars[row] = [blank; VGA_WIDTH];
    }

    pub fn newline(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let character = self.buffer_ptr.chars[row][col];
                self.buffer_ptr.chars[row - 1][col] = character;
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.column_pos = 0;
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::fmt(format_args!($($arg)*));
    };
}

pub fn fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| unsafe {
        WRITER.lock().write_fmt(args).ok();
    });
}
