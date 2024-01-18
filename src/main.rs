#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

mod multiboot;

global_asm!(include_str!("boot.asm"));

static mut VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

static PANIC_MSG: &[u8] = b"Panic!\0";
static HELLO: &[u8] = b"Hello World!";

#[panic_handler]
pub unsafe fn panic(_info: &PanicInfo) -> ! {
    PANIC_MSG.iter().enumerate().for_each(|(i, &byte)| unsafe {
        *VGA_BUFFER.offset(i as isize * 2) = byte;
        *VGA_BUFFER.offset(i as isize * 2 + 1) = 0xf;
    });
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn kernel_main() -> ! {
    HELLO.iter().enumerate().for_each(|(i, &byte)| unsafe {
        *VGA_BUFFER.offset(i as isize * 2) = byte;
        *VGA_BUFFER.offset(i as isize * 2 + 1) = 0xf;
    });

    loop {}
}
