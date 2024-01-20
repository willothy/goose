#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod boot_info;
mod idt;
mod pic;
mod vga;

use x86_64::instructions::interrupts;

#[panic_handler]
pub(crate) unsafe fn panic(info: &PanicInfo) -> ! {
    println!("Panic: ");
    if let Some(location) = info.location() {
        println!("{} at {}:", location.file(), location.line());
    }
    if let Some(message) = info.message().and_then(|m| m.as_str()) {
        println!("{}", message);
    } else if let Some(message) = info.payload().downcast_ref::<&str>() {
        println!("{}", message);
    } else if let Some(message) = info.payload().downcast_ref::<&[u8]>() {
        println!("{}", core::str::from_utf8(message).unwrap());
    } else {
        println!("unknown");
    }

    loop {}
}

extern "C" {
    fn load_gdt();
}

#[no_mangle]
pub extern "C" fn kernel_main(mboot_ptr: usize) -> ! {
    boot_info::init(mboot_ptr).expect("Failed to initialize boot info");

    println!("Hello from 64-bit Rust! Successfully entered long mode.");

    // Set up the IDT entries.
    idt::init();

    unsafe { load_gdt() };

    pic::init();

    interrupts::enable();

    println!("Interrupts enabled");

    loop {}
}
