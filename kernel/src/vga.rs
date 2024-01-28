use core::{
    mem::MaybeUninit,
    ptr::{addr_of, addr_of_mut, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use spin::{lazy::Lazy, Mutex, Once, RwLock};
use x86_64::instructions::interrupts::without_interrupts;

use crate::event::{push_event, UiEvent};

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

pub static mut BUFFER: *mut Buffer = 0xb8000 as *mut Buffer;

pub static mut WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| Mutex::new(Writer::new()));

const SCROLLBACK_LINE_COUNT: usize = VGA_HEIGHT * 10;
pub static mut SCROLLBACK_BUFFER: Lazy<[[ScreenChar; VGA_WIDTH]; SCROLLBACK_LINE_COUNT]> =
    Lazy::new(|| {
        [[ScreenChar {
            ascii_char: b' ',
            color_code: ColorCode::new(Color::White, Color::Black),
        }; VGA_WIDTH]; SCROLLBACK_LINE_COUNT]
    });
pub static mut SCROLLBACK: Lazy<Scrollback> = Lazy::new(|| Scrollback::new());

pub static SCROLL_TOPLINE: AtomicUsize = AtomicUsize::new(0);

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
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode,
}

pub struct Buffer {
    chars: [[ScreenChar; VGA_WIDTH]; VGA_HEIGHT],
}

impl Buffer {
    pub fn put(&mut self, row: usize, col: usize, character: ScreenChar) {
        self.chars[row][col] = character;
    }

    pub fn put_str(&mut self, row: usize, col: usize, s: &str, color: Option<ColorCode>) {
        let color = color.unwrap_or_else(|| ColorCode::new(Color::White, Color::Black));
        let mut col = col;
        let max_len = VGA_WIDTH - col;
        for byte in s.bytes().take(max_len) {
            self.put(
                row,
                col,
                ScreenChar {
                    ascii_char: byte,
                    color_code: color,
                },
            );
            col += 1;
        }
    }

    pub fn put_str_wrapped(
        &mut self,
        mut row: usize,
        col: usize,
        s: &str,
        color: Option<ColorCode>,
    ) {
        let color = color.unwrap_or_else(|| ColorCode::new(Color::White, Color::Black));
        let mut col = col;
        for byte in s.bytes() {
            self.put(
                row,
                col,
                ScreenChar {
                    ascii_char: byte,
                    color_code: color,
                },
            );
            col += 1;
            if col == VGA_WIDTH {
                col = 0;
                row += 1;
            }
        }
    }

    pub fn get(&self, row: usize, col: usize) -> ScreenChar {
        self.chars[row][col]
    }

    pub fn clear(&mut self, color: Option<ColorCode>) {
        let color = color.unwrap_or_else(|| ColorCode::new(Color::White, Color::Black));
        for row in 0..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                self.put(
                    row,
                    col,
                    ScreenChar {
                        ascii_char: b' ',
                        color_code: color,
                    },
                );
            }
        }
    }

    pub fn clear_row(&mut self, row: usize, color: Option<ColorCode>) {
        let color = color.unwrap_or_else(|| ColorCode::new(Color::White, Color::Black));
        self.chars[row] = [ScreenChar {
            ascii_char: b' ',
            color_code: color,
        }; VGA_WIDTH];
    }
}

pub struct Scrollback {
    buffer: *mut [[ScreenChar; VGA_WIDTH]; SCROLLBACK_LINE_COUNT],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl Scrollback {
    pub fn new() -> Self {
        Self {
            buffer: unsafe { SCROLLBACK_BUFFER.as_mut_ptr() },
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);
        if head >= tail {
            head - tail
        } else {
            head + SCROLLBACK_LINE_COUNT - tail
        }
    }

    pub fn get_line(&self, line: usize) -> &[ScreenChar; VGA_WIDTH] {
        let line = (self.tail.load(Ordering::SeqCst) + line) % SCROLLBACK_LINE_COUNT;
        unsafe { &(*self.buffer)[line] }
    }

    pub fn get_line_mut(&self, line: usize) -> &mut [ScreenChar; VGA_WIDTH] {
        let line = (self.tail.load(Ordering::SeqCst) + line) % SCROLLBACK_LINE_COUNT;
        unsafe { &mut (*self.buffer)[line] }
    }

    pub fn current_line_mut(&self) -> &mut [ScreenChar; VGA_WIDTH] {
        unsafe { &mut (*self.buffer)[self.head.load(Ordering::SeqCst)] }
    }

    pub fn next_line(&self) {
        // If the scrollback buffer is full, pop the oldest line
        let head = self.head.load(Ordering::SeqCst);

        self.head
            .store((head + 1) % SCROLLBACK_LINE_COUNT, Ordering::SeqCst);

        let tail = self.tail.load(Ordering::SeqCst);
        if head == tail {
            // self.overflow_offset += 1;
            self.tail
                .store((tail + 1) % SCROLLBACK_LINE_COUNT, Ordering::SeqCst);
        }

        // assert!(self.tail != self.head);

        // Auto-scroll if we're at the end of the buffer
        // if self.len() - SCROLL_TOPLINE.load(Ordering::SeqCst) < VGA_HEIGHT {
        //     SCROLL_TOPLINE.fetch_add(1, Ordering::SeqCst);
        // }
    }

    pub fn push(&mut self, line: [ScreenChar; VGA_WIDTH]) {
        let row = &mut unsafe { (*self.buffer)[self.head.load(Ordering::SeqCst)] };

        *row = line;
        self.next_line();
    }
}

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
}

impl Writer {
    pub fn new() -> Self {
        let color_code = ColorCode::new(Color::White, Color::Black);
        Self {
            column_pos: 0,
            color_code,
        }
    }

    pub fn scroll_up(&mut self) {
        SCROLL_TOPLINE
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |topline| {
                if topline > 0 {
                    Some(topline - 1)
                } else {
                    None
                }
            })
            .ok();
        self.draw_frame();
    }

    pub fn scroll_down(&mut self) {
        SCROLL_TOPLINE
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |topline| {
                if topline < unsafe { SCROLLBACK.len() } - VGA_HEIGHT {
                    Some(topline + 1)
                } else {
                    None
                }
            })
            .ok();
        self.draw_frame();
    }

    pub fn draw_frame(&mut self) {
        let topline = SCROLL_TOPLINE.load(Ordering::SeqCst);
        let start_line = unsafe { SCROLLBACK.tail.load(Ordering::SeqCst) } + topline; // + self.overflow_offset;
        let end_line = topline + VGA_HEIGHT - 1;

        for i in start_line..end_line {
            let i = i % SCROLLBACK_LINE_COUNT;
            let screenrow = i - start_line;
            let line = unsafe { (*SCROLLBACK.buffer)[i] };
            unsafe {
                (*BUFFER).chars[screenrow] = line;
            }
            // assert_eq!(i, screenrow);
        }
    }

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
                let column_pos = self.column_pos;
                let color_code = self.color_code;
                let scrollback_row = unsafe { SCROLLBACK.current_line_mut() };
                scrollback_row[column_pos] = ScreenChar {
                    ascii_char: byte,
                    color_code,
                };
                // self.buffer.put( VGA_HEIGHT - 1,
                //     self.column_pos,
                //     ScreenChar {
                //         ascii_char: byte,
                //         color_code: self.color_code,
                //     },
                // );
                self.column_pos += 1;
            }
        }
        self.draw_frame();
    }

    pub fn write_string(&mut self, s: &str) {
        s.bytes().for_each(|byte| match byte {
            0x20..=0x7e | b'\n' | b'\r' => self.write_byte(byte),
            _ => self.write_byte(0xfe),
        });
    }

    pub fn clear_row(&mut self, row: usize) {
        unsafe {
            (*BUFFER).clear_row(row, Some(self.color_code));
        }
    }

    pub fn newline(&mut self) {
        // for row in 1..VGA_HEIGHT {
        //     for col in 0..VGA_WIDTH {
        //         let character = self.buffer_ptr.chars[row][col];
        //         self.buffer_ptr.chars[row - 1][col] = character;
        //     }
        // }
        // self.clear_row(VGA_HEIGHT - 1);
        // self.scrollback_next_line();
        unsafe {
            SCROLLBACK.next_line();
        }
        self.column_pos = 0;
        self.draw_frame();
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

struct FmtWriter;

impl FmtWriter {}

impl core::fmt::Write for FmtWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        push_event(UiEvent::WriteStr(unsafe {
            (s as *const str).as_ref().unwrap()
        }));
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
        WRITER.lock().write_fmt(args).unwrap();
    });
}
