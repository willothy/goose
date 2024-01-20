use core::arch::asm;

use spin::once::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

static mut IDT: Once<InterruptDescriptorTable> = Once::new();

extern "x86-interrupt" fn divide_by_zero_handler(_stack_frame: InterruptStackFrame) {
    println!("Divide by zero");
}

extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    println!("Timer interrupt: {:#?}", stack_frame);
}

extern "x86-interrupt" fn default_handler() {
    println!("Unhandled interrupt");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub fn init() {
    unsafe {
        IDT.call_once(|| InterruptDescriptorTable::new());
    }
    let idt = get_mut();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.divide_error.set_handler_fn(divide_by_zero_handler);

    idt[32].set_handler_fn(timer_handler);

    idt.load();

    // // Set up the IDT entries.
    // idt::get_entry_mut(0).set_handler(divide_by_zero_handler);
    // idt::get_entry_mut(32).set_handler(timer_handler);

    // for i in 0..32 {
    //     unsafe {
    //         IDT.entries[i].set_handler(default_handler);
    //     }
    // }
}

pub fn get() -> &'static InterruptDescriptorTable {
    unsafe { IDT.get().expect("IDT not initialized") }
}

pub fn get_mut() -> &'static mut InterruptDescriptorTable {
    unsafe { IDT.get_mut().expect("IDT not initialized") }
}
