#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

// global_asm!(include_str!("boot.asm"));

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT * 2; // each cell is 2 bytes
static mut VGA_BUFFER: *mut [u8; VGA_BUFFER_SIZE] = 0xb8000 as *mut [u8; VGA_BUFFER_SIZE];

static PANIC_MSG: &[u8] = b"Panic!\0";
static HELLO: &[u8] = b"Hello World!";

static mut LAST_ROW: usize = 0;
static mut LAST_COL: usize = 0;

fn print(s: &[u8]) {
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
                (*VGA_BUFFER)[offset] = byte;
                (*VGA_BUFFER)[offset + 1] = 0xf;
                LAST_COL += 1;
            }
        }
    });
}

fn println(s: &[u8]) {
    print(s);
    unsafe {
        LAST_ROW += 1;
        LAST_COL = 0;
    }
}

fn num_to_char(n: u8) -> u8 {
    match n {
        0..=9 => n + 48,
        10..=15 => n + 87,
        _ => panic!(),
    }
}

fn print_int(n: u32) {
    let mut n = n;
    let mut digits = [b' '; 10];
    let mut i = 0;
    while n > 0 {
        digits[i] = num_to_char((n % 10) as u8);
        n /= 10;
        i += 1;
    }
    digits.reverse();
    print(b"0x");
    println(&digits[..])
}

#[panic_handler]
pub unsafe fn panic(_info: &PanicInfo) -> ! {
    print(PANIC_MSG);
    loop {}
}

unsafe fn get_multiboot_info() -> *const u8 {
    let ptr: *const u8;
    asm!(
        "mov {}, eax",
        out(reg) ptr
    );
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn kernel_main() -> ! {
    // let ch: *mut u16 = VGA_BUFFER as *mut [u8; VGA_BUFFER_SIZE] as *mut u16;
    // *ch = 0x480f;
    // (*VGA_BUFFER)[0] = b'H';
    // (*VGA_BUFFER)[1] = 0xf;
    // print(HELLO);
    // print(HELLO);
    // print(HELLO);
    // print(HELLO);
    // println(HELLO);

    loop {}
}
