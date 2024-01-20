use spin::once::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

static mut IDT: Once<InterruptDescriptorTable> = Once::new();

macro_rules! handler {
    ($name:ident: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame);
        }
    };
    (!$name:ident: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, error: u64) -> ! {
            $body(stack_frame, error);
            loop {}
        }
    };
}

macro_rules! handlers {
    { $name:ident: $body:expr; $($rest:tt)* } => {
        handler!($name: $body);
        handlers!($($rest)*);
    };
    { $name:ident: $body:expr; } => {
        handler!($name: $body);
    };
    { ! $name:ident: $body:expr; } => {
        handler!(!$name: $body);
    };
    { ! $name:ident: $body:expr; $($rest:tt)* } => {
        handler!(!$name: $body);
        handlers!($($rest)*);
    };
    {} => {}
}

handlers! {
    divide: |_| {
        println!("Divide by zero");
    };
    breakpoint: |stack_frame| {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    };
    timer: |stack_frame| {
        println!("Timer interrupt: {:#?}", stack_frame);
    };
    !double_fault: |stack_frame, _error_code| {
        println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
        loop {}
    };
}

pub fn init() {
    unsafe {
        IDT.call_once(|| InterruptDescriptorTable::new());
    }
    let idt = get_mut();
    idt.breakpoint.set_handler_fn(breakpoint);
    idt.divide_error.set_handler_fn(divide);
    idt.double_fault.set_handler_fn(double_fault);

    idt[32].set_handler_fn(timer);

    idt.load();
}

#[allow(dead_code)]
pub fn get() -> &'static InterruptDescriptorTable {
    unsafe { IDT.get().expect("IDT not initialized") }
}

pub fn get_mut() -> &'static mut InterruptDescriptorTable {
    unsafe { IDT.get_mut().expect("IDT not initialized") }
}
