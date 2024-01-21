use spin::lazy::Lazy;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

use crate::{gdt, pic, print, println};

#[repr(u8)]
pub enum InterruptIndex {
    Timer = pic::PIC_1_OFFSET,
    Keyboard,
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, _error: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    pic::end_interrupt(InterruptIndex::Timer as u8);
}

extern "x86-interrupt" fn keyboard(_stack_frame: InterruptStackFrame) {
    let mut port: Port<u8> = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    let key = match scancode {
        0x02..=0x0b => Some(b"1234567890"[scancode as usize - 0x02] as char),
        0x10..=0x19 => Some(b"qwertyuiop"[scancode as usize - 0x10] as char),

        0x1e..=0x26 => Some(b"asdfghjkl"[scancode as usize - 0x1e] as char),
        0x2c..=0x32 => Some(b"zxcvbnm"[scancode as usize - 0x2c] as char),
        0x39 => Some(' '),
        0xF => Some(' '), // Tab
        _ => None,
    };
    if let Some(key) = key {
        print!("{}", key);
    }
    // else {
    //     println!("Unknown key: 0x{:0X}", scancode);
    // }
    pic::end_interrupt(InterruptIndex::Keyboard as u8);
}

extern "x86-interrupt" fn divide_handler(_stack_frame: InterruptStackFrame) {
    println!("Divide by zero");
}

static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.divide_error.set_handler_fn(divide_handler);
    idt.breakpoint.set_handler_fn(breakpoint);
    idt[InterruptIndex::Timer as usize].set_handler_fn(timer);
    idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    idt
});

pub fn init() {
    unsafe {
        IDT.load();
    }
}
