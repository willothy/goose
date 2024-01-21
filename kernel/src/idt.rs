use spin::lazy::Lazy;
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
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

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer(_stack_frame: InterruptStackFrame) {
    print!(".");
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

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    println!("Debug");
}

extern "x86-interrupt" fn non_maskable_interrupt_handler(_stack_frame: InterruptStackFrame) {
    println!("Non-maskable interrupt");
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: InterruptStackFrame) {
    println!("Overflow");
}

extern "x86-interrupt" fn bound_range_exceeded_handler(_stack_frame: InterruptStackFrame) {
    println!("Bound range exceeded");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    println!("Invalid opcode");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: InterruptStackFrame) {
    println!("Device not available");
}

extern "x86-interrupt" fn invalid_tss_handler(_stack_frame: InterruptStackFrame, _error: u64) {
    println!("Invalid TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: InterruptStackFrame,
    _error: u64,
) {
    println!("Segment not present");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error: u64,
) {
    println!("Stack segment fault");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    _error: u64,
) {
    println!("General protection fault: {:?}", stack_frame);
    // pic::end_interrupt(13);
    loop {
        // x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    _error: PageFaultErrorCode,
) {
    println!("Page fault: {:?}", stack_frame);
    loop {
        // x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn x87_floating_point_handler(_stack_frame: InterruptStackFrame) {
    println!("x87 floating point");
}

extern "x86-interrupt" fn alignment_check_handler(_stack_frame: InterruptStackFrame, _error: u64) {
    println!("Alignment check");
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) -> ! {
    println!("Machine check");
    loop {
        // x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn simd_floating_point_handler(_stack_frame: InterruptStackFrame) {
    println!("SIMD floating point");
}

extern "x86-interrupt" fn virtualization_handler(_stack_frame: InterruptStackFrame) {
    println!("Virtualization");
}

extern "x86-interrupt" fn security_exception_handler(
    _stack_frame: InterruptStackFrame,
    _error: u64,
) {
    println!("Security exception");
}

static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.divide_error.set_handler_fn(divide_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.debug.set_handler_fn(debug_handler);
    idt.non_maskable_interrupt
        .set_handler_fn(non_maskable_interrupt_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.bound_range_exceeded
        .set_handler_fn(bound_range_exceeded_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available
        .set_handler_fn(device_not_available_handler);
    idt.double_fault.set_handler_fn(double_fault);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.x87_floating_point
        .set_handler_fn(x87_floating_point_handler);
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    idt.machine_check.set_handler_fn(machine_check_handler);
    idt.simd_floating_point
        .set_handler_fn(simd_floating_point_handler);
    idt.virtualization.set_handler_fn(virtualization_handler);
    idt.security_exception
        .set_handler_fn(security_exception_handler);

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
