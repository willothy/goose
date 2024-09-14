#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::{arch::asm, panic::PanicInfo};

use x86_64::instructions::interrupts;

mod boot_info;
mod debug;
mod gdt;
mod idt;
mod mem;
mod pic;
mod pit;
mod vga;

#[panic_handler]
pub(crate) unsafe fn panic(info: &PanicInfo) -> ! {
    println!("Panic: ");
    if let Some(location) = info.location() {
        println!("{} at {}:", location.file(), location.line());
    }
    if let Some(message) = info.message().as_str() {
        println!("{}", message);
    } else {
        println!("unknown");
    }

    loop {
        x86_64::instructions::hlt();
    }
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

pub extern "C" fn user_mode_entry() -> ! {
    let cs: u16;
    unsafe {
        asm! {
            "mov ax, cs",
            out("ax") cs,
        };
    }
    if (cs & 0b11) != 0b11 {
        panic!("Not in ring 3!");
    }
    println!("Hello from user mode!");
    loop {}
}

const KERNEL_STACK_SIZE: usize = 8 * 1024;
static mut KERNEL_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

#[no_mangle]
pub extern "C" fn kernel_main(mboot_ptr: usize) -> ! {
    // Set up the stack for the kernel.
    unsafe {
        asm! {
            "mov rsp, {stack}",
            stack =  in(reg) core::ptr::addr_of_mut!(KERNEL_STACK),
        };
    }

    // Initialize the boot info so that we can use it as needed with a 'static lifetime.
    boot_info::init(mboot_ptr).expect("Failed to initialize boot info");

    // Parse the memory map that the bootloader (hopefully) provided.
    mem::find_available_regions();

    println!("Hello from 64-bit Rust! Successfully entered long mode.");

    // Set up the GDT.
    gdt::init();

    // Set up the IDT entries.
    idt::init();

    // Setup interrupt timer, 10ms preempt by default.
    pit::init();

    // Setup the PIC.
    pic::init();

    // This will be done later once we enter user mode.
    interrupts::enable();
    println!("Interrupts enabled");

    let info = boot_info::boot_info();

    println!("Loaded by {}", info.loader);
    println!("Command line: {:?}", info.cmdline);

    // let selectors = gdt::selectors();
    // let mut tss = selectors.tss.0;
    // let mut cs = selectors.ring3_code.0;
    //
    // println!("TSS: {:x}", tss);
    // println!("CS: {:x}", cs);

    // Jump to user mode. Not ready to do this yet.
    // unsafe {
    //     asm! {
    //         // "push {:x}",
    //         "push 0x18|3",
    //         "push rsp",
    //         // "push 0x7c00",
    //         "push 0x2",
    //         // "push {:x}",
    //         "push 0x10|3",
    //         "push {user_mode_entry}",
    //         "iretq",
    //         // in(reg) tss,
    //         // in(reg) cs,
    //         user_mode_entry = in(reg) user_mode_entry,
    //     };
    // }

    // #[cfg(test)]
    // test_main();

    loop {
        x86_64::instructions::hlt();
    }
}
