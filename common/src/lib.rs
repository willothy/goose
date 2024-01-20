#![no_std]

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;
pub const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT * 2; // each cell is 2 bytes

pub static mut LAST_ROW: usize = 0;
pub static mut LAST_COL: usize = 0;

pub fn print(s: &[u8]) {
    let buf = 0xb8000 as *mut u8;
    s.iter().for_each(|&byte| unsafe {
        if LAST_ROW >= VGA_HEIGHT {
            LAST_ROW = 0;
        }
        if LAST_COL >= VGA_WIDTH {
            LAST_COL = 0;
        }
        match byte {
            b'\n' => {
                LAST_ROW += 1;
                LAST_COL = 0;
            }
            b'\r' => {
                LAST_COL = 0;
            }
            _ => {
                let offset = ((80 * LAST_ROW) + LAST_COL) * 2;
                *buf.add(offset) = byte;
                *buf.add(offset + 1) = 0xf;
                LAST_COL += 1;
            }
        }
    });
}

pub fn int_to_char(n: u8) -> u8 {
    match n {
        0..=9 => n + 48,
        10..=15 => n + 87,
        _ => panic!(),
    }
}

pub fn newline() {
    unsafe {
        LAST_ROW += 1;
        LAST_COL = 0;
    }
}

pub fn println(s: &[u8]) {
    print(s);
    newline();
}

pub fn print_int(mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[9 - i] = int_to_char((n % 10) as u8);
        n /= 10;
        i += 1;
    }
    print(&buf[10 - i..]);
}

pub fn print_hex(n: u32) {
    let mut buf = [0u8; 8];
    let mut i = 0;
    let mut n = n;
    while n > 0 {
        buf[7 - i] = int_to_char((n % 16) as u8);
        n /= 16;
        i += 1;
    }
    print(b"0x");
    print(&buf[8 - i..]);
}

pub fn print_hex_64(n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;
    while n > 0 {
        buf[19 - i] = int_to_char((n % 16) as u8);
        n /= 16;
        i += 1;
    }
    print(b"0x");
    print(&buf[20 - i..]);
}
