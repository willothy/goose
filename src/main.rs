#![no_std]
#![no_main]

use core::{
    mem::{size_of, transmute},
    panic::PanicInfo,
};

static PANIC_MSG: &[u8] = b"Panic!\0";
static HELLO: &[u8] = b"Hello from 64-bit Rust! Successfully entered long mode.";

pub fn print(s: &[u8]) {
    static mut LAST_ROW: usize = 0;
    static mut LAST_COL: usize = 0;
    let buf = 0xb8000 as *mut u8;
    s.iter().for_each(|&byte| unsafe {
        if LAST_ROW >= 24 {
            LAST_ROW = 0;
        }
        if LAST_COL >= 80 {
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

#[panic_handler]
pub unsafe fn panic(_info: &PanicInfo) -> ! {
    print(PANIC_MSG);
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn kernel_main() -> ! {
    *(0xb8000 as *mut u8) = 'F' as u8;
    *(0xb8001 as *mut u8) = 0xf;

    *(0xb8000 as *mut u8) = '0' as u8;
    *(0xb8001 as *mut u8) = 0xf;
    *(0xb8002 as *mut u8) = 'x' as u8;
    *(0xb8003 as *mut u8) = 0xf;
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = size_of::<usize>();
    // while n > 0 && i < 5 {
    //     buf[i] = match (n % 10) as u8 {
    //         c @ 0..=9 => c + 48,
    //         c @ 10..=15 => c + 87,
    //         _ => panic!(),
    //     };
    //     n /= 10;
    //     i += 1;
    // }
    // for i in 0..19 {
    //     let byte = (0xb8000 + (i * 2)) as *mut u8;
    //     *byte = buf[18 - i];
    //     let byte = (0xb8001 + (i * 2)) as *mut u8;
    //     *byte = 0xf;
    // }

    // static mut LAST_ROW: usize = 0;
    // static mut LAST_COL: usize = 0;
    // let mut row = 0;
    // let mut col = 0;
    // b"Hello".iter().for_each(|byte| unsafe {
    //     if row >= 24 {
    //         row = 0;
    //     }
    //     if col >= 80 {
    //         col = 0;
    //     }
    //     match byte {
    //         b'\n' => {
    //             row += 1;
    //             col = 0;
    //         }
    //         b'\r' => {
    //             col = 0;
    //         }
    //         _ => {
    //             let offset = ((80 * row) + col) * 2;
    //             *((0xb8000 + offset) as *mut u8) = *byte;
    //             *((0xb8001 + offset) as *mut u8) = 0xf;
    //             col += 1;
    //         }
    //     }
    // });
    loop {}
}
