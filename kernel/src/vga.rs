use core::{
    mem::MaybeUninit,
    ptr::{addr_of, addr_of_mut, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use spin::{lazy::Lazy, Mutex, Once, RwLock};
use x86_64::instructions::interrupts::without_interrupts;

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

const SCROLLBACK_LINE_COUNT: usize = VGA_HEIGHT * 10;
pub static mut SCROLLBACK: [[ScreenChar; VGA_WIDTH]; SCROLLBACK_LINE_COUNT] = [[ScreenChar {
    ascii_char: b' ',
    color_code: ColorCode::new(Color::White, Color::Black),
}; VGA_WIDTH];
    SCROLLBACK_LINE_COUNT];
pub static mut WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| Mutex::new(Writer::new()));
pub static mut QUEUE_BUFFER: MaybeUninit<[UiEvent; 128]> = MaybeUninit::zeroed();
pub static mut UI_EVT_QUEUE: Lazy<UiEventQueue> = Lazy::new(|| UiEventQueue {
    events: unsafe { NonNull::new_unchecked(addr_of_mut!(QUEUE_BUFFER).cast()) },
    head: AtomicUsize::new(0),
    tail: AtomicUsize::new(0),
});

pub fn push_event(event: UiEvent) {
    unsafe { UI_EVT_QUEUE.push(event) };
}

pub fn pop_event() -> Option<UiEvent> {
    unsafe { UI_EVT_QUEUE.pop() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    ScrollDown,
    ScrollUp,
    WriteStr(&'static str),
}

const UI_QUEUE_SIZE: usize = 128;

pub struct UiEventQueue {
    pub events: NonNull<[UiEvent; UI_QUEUE_SIZE]>,
    pub head: AtomicUsize,
    pub tail: AtomicUsize,
}

impl UiEventQueue {
    pub fn push(&self, event: UiEvent) {
        without_interrupts(|| {
            let mut events = self.events;
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            if (tail + 1) % UI_QUEUE_SIZE == head {
                return;
            }
            unsafe { events.as_mut()[tail] = event };
            self.tail
                .store((tail + 1) % UI_QUEUE_SIZE, Ordering::SeqCst);
        });
    }

    pub fn pop(&self) -> Option<UiEvent> {
        without_interrupts(|| {
            let events = self.events;
            let head = self.head.load(Ordering::SeqCst);
            let tail = self.tail.load(Ordering::SeqCst);
            if head == tail {
                return None;
            }
            let event = unsafe { events.as_ref()[head].clone() };
            self.head
                .store((head + 1) % UI_QUEUE_SIZE, Ordering::SeqCst);
            Some(event)
        })
    }
}

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

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    scrollback: &'static mut [[ScreenChar; VGA_WIDTH]; SCROLLBACK_LINE_COUNT],
    scrollback_head: usize,
    scrollback_tail: usize,
    overflow_offset: usize,
    scroll_topline: usize,
}

impl Writer {
    pub fn new() -> Self {
        let color_code = ColorCode::new(Color::White, Color::Black);
        Self {
            column_pos: 0,
            color_code,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            scrollback: unsafe { &mut *addr_of_mut!(SCROLLBACK) },
            scrollback_head: 0,
            scrollback_tail: 0,
            overflow_offset: 0,
            scroll_topline: 0,
        }
    }

    pub fn scrollback_len(&self) -> usize {
        if self.scrollback_head >= self.scrollback_tail {
            self.scrollback_head - self.scrollback_tail
        } else {
            self.scrollback_head + self.scrollback.len() - self.scrollback_tail
        }
    }

    pub fn scrollback_get_line(&self, line: usize) -> &[ScreenChar; VGA_WIDTH] {
        let line = (self.scrollback_tail + line) % self.scrollback.len();
        &self.scrollback[line]
    }

    pub fn scrollback_get_line_mut(&mut self, line: usize) -> &mut [ScreenChar; VGA_WIDTH] {
        let line = (self.scrollback_tail + line) % self.scrollback.len();
        &mut self.scrollback[line]
    }

    pub fn scrollback_current_line_mut(&mut self) -> &mut [ScreenChar; VGA_WIDTH] {
        &mut self.scrollback[self.scrollback_head]
    }

    pub fn scrollback_next_line(&mut self) {
        // If the scrollback buffer is full, pop the oldest line
        self.scrollback_head = (self.scrollback_head + 1) % SCROLLBACK_LINE_COUNT;

        if self.scrollback_head == self.scrollback_tail {
            self.overflow_offset += 1;
            self.scrollback_tail = (self.scrollback_tail + 1) % SCROLLBACK_LINE_COUNT;
        }

        // assert!(self.scrollback_tail != self.scrollback_head);

        // Auto-scroll if we're at the end of the buffer
        // if self.scrollback_len() - self.scroll_topline < VGA_HEIGHT {
        //     self.scroll_topline += 1;
        // }
    }

    pub fn scrollback_push(&mut self, line: [ScreenChar; VGA_WIDTH]) {
        let row = &mut self.scrollback[self.scrollback_head];

        *row = line;
        self.scrollback_next_line();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_topline > 0 {
            self.scroll_topline -= 1;
        }
        self.draw_frame();
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_topline < self.scrollback_len() - VGA_HEIGHT {
            self.scroll_topline += 1;
        }
        self.draw_frame();
    }

    pub fn draw_frame(&mut self) {
        let topline = self.scroll_topline;
        let start_line = self.scrollback_tail + topline; // + self.overflow_offset;
        let end_line = topline + VGA_HEIGHT - 1;

        for i in start_line..end_line {
            // let i = i % SCROLLBACK_LINE_COUNT;
            let screenrow = i - start_line;
            let line = self.scrollback[i];
            self.buffer.chars[screenrow] = line;
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
                let scrollback_row = self.scrollback_current_line_mut();
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
        self.buffer.clear_row(row, Some(self.color_code));
    }

    pub fn newline(&mut self) {
        // for row in 1..VGA_HEIGHT {
        //     for col in 0..VGA_WIDTH {
        //         let character = self.buffer_ptr.chars[row][col];
        //         self.buffer_ptr.chars[row - 1][col] = character;
        //     }
        // }
        // self.clear_row(VGA_HEIGHT - 1);
        self.scrollback_next_line();
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
