#![no_std]
#![no_main]
#![feature(panic_info_message)]
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
    if let Some(message) = info.message().and_then(|m| m.as_str()) {
        println!("{}", message);
    } else if let Some(message) = info.payload().downcast_ref::<&str>() {
        println!("{}", message);
    } else if let Some(message) = info.payload().downcast_ref::<&[u8]>() {
        println!("{}", core::str::from_utf8(message).unwrap());
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

#[no_mangle]
pub extern "C" fn kernel_main(mboot_ptr: usize) -> ! {
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

    // boot_info::dump();

    // let selectors = gdt::selectors();
    // let mut tss = selectors.tss.0;
    // let mut cs = selectors.ring3_code.0;

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
        interrupts::without_interrupts(|| {
            let Some(evt) = crate::vga::pop_event() else {
                return;
            };
            println!("got event: {:?}", evt);
            match evt {
                crate::vga::UiEvent::ScrollUp => unsafe {
                    // if !crate::vga::WRITER.is_locked() {
                    //     crate::vga::WRITER.lock().scroll_up();
                    // }
                    // crate::vga::WRITER.lock().scroll_up();
                },
                crate::vga::UiEvent::ScrollDown => unsafe {
                    // if !crate::vga::WRITER.is_locked() {
                    //     crate::vga::WRITER.lock().scroll_down();
                    // }
                    // crate::vga::WRITER.lock().scroll_down();
                },
                crate::vga::UiEvent::WriteStr(str) => unsafe {
                    // println!("write str: {}", str);
                    // crate::vga::WRITER.lock().write_string(str);
                },
            }
        });

        x86_64::instructions::hlt();
    }
}
