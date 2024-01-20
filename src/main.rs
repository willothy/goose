#![no_std]
#![no_main]

use core::panic::PanicInfo;

use common::*;

static PANIC_MSG: &[u8] = b"Panic!\0";
static HELLO: &[u8] = b"Hello from 64-bit Rust! Successfully entered long mode.";

#[panic_handler]
pub unsafe fn panic(_info: &PanicInfo) -> ! {
    print(PANIC_MSG);
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn kernel_main() -> ! {
    print(HELLO);
    loop {}
}
