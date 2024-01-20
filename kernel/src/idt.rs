use spin::lazy::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{
    gdt, idt,
    pic::{self, end_interrupt},
    println,
};

macro_rules! handler {
    ($idt:expr, $name:ident: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame);
        }
        $idt.$name.set_handler_fn($name);
    };
    ($idt:expr, !$name:ident: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, error: u64) -> ! {
            $body(stack_frame, error);
            loop {}
        }
        $idt.$name.set_handler_fn($name);
    };
    ($idt:expr, $name:ident [$id:expr]: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame) {
            $body(stack_frame, $id);
        }
        $idt[$id as usize].set_handler_fn($name);
    };
    ($idt:expr, !$name:ident [$id:expr]: $body:expr) => {
        extern "x86-interrupt" fn $name(stack_frame: InterruptStackFrame, error: u64) -> ! {
            $body(stack_frame, $id, error);
            loop {}
        }
        $idt[$id as usize].set_handler_fn($name);
    };
    () => {};
}

macro_rules! handlers {
    {$idt:expr, $name:ident: $body:expr; $($rest:tt)* } => {
        handler!($idt, $name: $body);
        handlers!($idt, $($rest)*);
    };
    {$idt:expr,  $name:ident: $body:expr; } => {
        handler!($idt, $name: $body);
    };
    {$idt:expr,  ! $name:ident: $body:expr; } => {
        handler!($idt, !$name: $body);
    };
    {$idt:expr,  ! $name:ident: $body:expr; $($rest:tt)* } => {
        handler!($idt, !$name: $body);
        handlers!($idt, $($rest)*);
    };
    {$idt:expr, $name:ident [$id:expr]: $body:expr; $($rest:tt)+ } => {
        handler!($idt, $name [$id]: $body);
        handlers!($idt, $($rest)*);
    };
    {$idt:expr,  $name:ident [$id:expr]: $body:expr; } => {
        handler!($idt, $name [$id]: $body);
    };
    {$idt:expr,  ! $name:ident [$id:expr]: $body:expr; } => {
        handler!($idt, !$name [$id]: $body);
    };
    {$idt:expr,  ! $name:ident [$id:expr]: $body:expr; $($rest:tt)* } => {
        handler!($idt, !$name [$id]: $body);
        handlers!($idt, $($rest)*);
    };
    {} => {}
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, _error: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    handlers! {
        idt,
        divide_error: |_| {
            println!("Divide by zero");
        };
        breakpoint: |stack_frame| {
            println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
        };
        timer[pic::pic_1::OFFSET]: |_stack_frame, this| {
            end_interrupt(this);
        };
    }
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
