#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;

mod boot_info;
mod gdt;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[no_mangle]
pub extern "C" fn kernel_main(mboot_ptr: usize) -> ! {
    boot_info::init(mboot_ptr).expect("Failed to initialize boot info");

    println!("Hello from 64-bit Rust! Successfully entered long mode.");

    // Set up the GDT.
    gdt::init();

    // Set up the IDT entries.
    idt::init();

    pic::init();

    interrupts::enable();

    println!("Interrupts enabled");

    #[cfg(test)]
    test_main();

    loop {}
}
