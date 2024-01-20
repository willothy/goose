#![no_std]
#![no_main]

use core::panic::PanicInfo;

static PANIC_MSG: &[u8] = b"Panic!\0";
static HELLO: &[u8] = b"Hello from 64-bit Rust! Successfully entered long mode.";

mod multiboot_header;

use common::*;

#[panic_handler]
pub(crate) unsafe fn panic(_info: &PanicInfo) -> ! {
    print(PANIC_MSG);
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    print(HELLO);
    loop {}
}
