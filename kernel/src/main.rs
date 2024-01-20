#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]

use core::{
    arch::asm,
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

static PANIC_MSG: &[u8] = b"Panic";
static HELLO: &[u8] = b"Hello from 64-bit Rust! Successfully entered long mode.\n";

mod idt;
mod multiboot_header;
mod pic;

use common::*;
use idt::enable_interrupts;

#[panic_handler]
pub(crate) unsafe fn panic(info: &PanicInfo) -> ! {
    print(PANIC_MSG);
    if let Some(location) = info.location() {
        print(b" at ");
        print(location.file().as_bytes());
        print(b":");
        print_int(location.line());
    }
    if let Some(message) = info.message().and_then(|m| m.as_str()) {
        print(b": ");
        print(message.as_bytes());
    } else if let Some(message) = info.payload().downcast_ref::<&str>() {
        print(b": ");
        print(message.as_bytes());
    } else if let Some(message) = info.payload().downcast_ref::<&[u8]>() {
        print(b": ");
        print(message);
    } else {
        print(b".");
    }
    newline();

    loop {}
}

extern "C" fn divide_by_zero_handler() {
    // panic!("Divide by zero");
    print(b"Divide by zero");
    // unsafe {
    //     asm!("iretq");
    // }
    loop {}
}

extern "C" fn timer_handler() {
    let registers = unsafe { save_reg_states() };

    print(b"T");

    unsafe {
        restore_reg_states(registers);
        asm!("iretq");
    }
}

extern "C" {
    fn load_gdt();
}

pub struct Registers {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

#[inline(always)]
/// Safety: very much not safe.
pub unsafe fn save_reg_states() -> Registers {
    let rax: u64;
    let rbx: u64;
    let rcx: u64;
    let rdx: u64;
    let rsi: u64;
    let rdi: u64;
    let rbp: u64;
    let r8: u64;
    let r9: u64;
    let r10: u64;
    let r11: u64;
    let r12: u64;
    let r13: u64;
    let r14: u64;
    let r15: u64;

    asm!(
        "mov {}, rax",
        "mov {}, rbx",
        "mov {}, rcx",
        "mov {}, rdx",
        "mov {}, rsi",
        "mov {}, rdi",
        "mov {}, rbp",
        "mov {}, r8",
        "mov {}, r9",
        "mov {}, r10",
        "mov {}, r11",
        "mov {}, r12",
        "mov {}, r13",
        "mov {}, r14",
        "mov {}, r15",
        out(reg) rax,
        out(reg) rbx,
        out(reg) rcx,
        out(reg) rdx,
        out(reg) rsi,
        out(reg) rdi,
        out(reg) rbp,
        out(reg) r8,
        out(reg) r9,
        out(reg) r10,
        out(reg) r11,
        out(reg) r12,
        out(reg) r13,
        out(reg) r14,
        out(reg) r15,
    );

    Registers {
        rax,
        rbx,
        rcx,
        rdx,
        rsi,
        rdi,
        rbp,
        r8,
        r9,
        r10,
        r11,
        r12,
        r13,
        r14,
        r15,
    }
}

#[inline(always)]
/// Safety: very much not safe.
pub unsafe fn restore_reg_states(
    Registers {
        rax,
        rbx,
        rcx,
        rdx,
        rsi,
        rdi,
        rbp,
        r8,
        r9,
        r10,
        r11,
        r12,
        r13,
        r14,
        r15,
    }: Registers,
) {
    asm!(
        "mov rax, {}",
        "mov rbx, {}",
        "mov rcx, {}",
        "mov rdx, {}",
        "mov rsi, {}",
        "mov rdi, {}",
        "mov rbp, {}",
        "mov r8,  {}",
        "mov r9,  {}",
        "mov r10, {}",
        "mov r11, {}",
        "mov r12, {}",
        "mov r13, {}",
        "mov r14, {}",
        "mov r15, {}",
        in(reg) rax,
        in(reg) rbx,
        in(reg) rcx,
        in(reg) rdx,
        in(reg) rsi,
        in(reg) rdi,
        in(reg) rbp,
        in(reg) r8,
        in(reg) r9,
        in(reg) r10,
        in(reg) r11,
        in(reg) r12,
        in(reg) r13,
        in(reg) r14,
        in(reg) r15
    );
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    print(HELLO);

    // Set up the IDT ptr.
    idt::init();

    // Set up the IDT entries.
    idt::get_entry_mut(0).set_handler(divide_by_zero_handler);
    idt::get_entry_mut(32).set_handler(timer_handler);

    unsafe {
        load_gdt();
    }

    // Load the IDT.
    idt::load();

    pic::init();

    unsafe {
        enable_interrupts();
    }

    print(b"Interrupts enabled\n");
    loop {}
}
