use core::ops::{Deref, DerefMut, Index, IndexMut};

use spin::lazy::Lazy;
use x86_64::{
    instructions::port::Port,
    structures::idt::{
        Entry, HandlerFunc, InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
    },
};

use crate::{
    event::{self, UiEvent},
    gdt, pic, print, println,
};

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptIndex {
    // Hardware-defined interrupts
    Divide = 0,
    Debug = 1,
    NonMaskable = 2,
    Breakpoint = 3,
    Overflow = 4,
    BoundRangeExceeded = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    CoprocessorSegmentOverrun = 9, // reserved
    InvalidTss = 10,
    SegmentNotPresent = 11,
    StackSegmentFault = 12,
    GeneralProtectionFault = 13,
    PageFault = 14,
    Reserved15 = 15, // reserved
    X87FloatingPoint = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SimdFloatingPoint = 19,
    Virtualization = 20,
    CPProtectionException = 21,
    Reserved22 = 22, // reserved
    Reserved23 = 23, // reserved
    Reserved24 = 24, // reserved
    Reserved25 = 25, // reserved
    Reserved26 = 26, // reserved
    Reserved27 = 27, // reserved
    HypervisorInjectionException = 28,
    VMMCommunicationException = 29,
    SecurityException = 30,
    Reserved31 = 31, // reserved

    // Interrupts
    Timer = pic::PIC_1_OFFSET,
    Keyboard,
    MaybeSpurious = 39,
}

pub struct IdtBuilder(InterruptDescriptorTable);

impl IdtBuilder {
    pub fn new() -> Self {
        Self(InterruptDescriptorTable::new())
    }

    pub fn into_inner(self) -> InterruptDescriptorTable {
        self.0
    }
}

impl Deref for IdtBuilder {
    type Target = InterruptDescriptorTable;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IdtBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<InterruptIndex> for IdtBuilder {
    type Output = Entry<HandlerFunc>;

    fn index(&self, index: InterruptIndex) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<InterruptIndex> for IdtBuilder {
    fn index_mut(&mut self, index: InterruptIndex) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, _error: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
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

        // 0x1e..=0x26 => Some(b"asdfghjkl"[scancode as usize - 0x1e] as char),
        0x1e => Some('a'),
        0x1f => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => {
            event::push_event(UiEvent::ScrollDown);
            Some('j')
            // None
        }
        0x25 => {
            event::push_event(UiEvent::ScrollUp);
            Some('k')
            // None
        }
        0x26 => Some('l'),
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
    // println!("Segment not present");
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error: u64,
) {
    // println!("Stack segment fault");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error: u64,
) {
    println!("General protection fault: {:?}", stack_frame);
    println!("Error code: {}", error);
    // pic::end_interrupt(13);
    loop {
        //     // x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    _error: PageFaultErrorCode,
) {
    // println!("Page fault: {:?}", stack_frame);
    // loop {
    //     // x86_64::instructions::hlt();
    // }
}

extern "x86-interrupt" fn x87_floating_point_handler(_stack_frame: InterruptStackFrame) {
    // println!("x87 floating point");
}

extern "x86-interrupt" fn alignment_check_handler(_stack_frame: InterruptStackFrame, _error: u64) {
    // println!("Alignment check");
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) -> ! {
    // println!("Machine check");
    loop {
        // x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn simd_floating_point_handler(_stack_frame: InterruptStackFrame) {
    // println!("SIMD floating point");
}

extern "x86-interrupt" fn virtualization_handler(_stack_frame: InterruptStackFrame) {
    // println!("Virtualization");
}

extern "x86-interrupt" fn security_exception_handler(
    _stack_frame: InterruptStackFrame,
    _error: u64,
) {
    // println!("Security exception");
}

extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if (pic::read_isr() & (1 << 7)) != 0 {
        // Spurious interrupt
        pic::end_interrupt(InterruptIndex::MaybeSpurious as u8);
    }
}

static mut IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    use InterruptIndex::*;
    let mut idt = IdtBuilder::new();

    idt[Divide].set_handler_fn(divide_handler);
    idt[Breakpoint].set_handler_fn(breakpoint_handler);
    idt[Debug].set_handler_fn(debug_handler);
    idt[NonMaskable].set_handler_fn(non_maskable_interrupt_handler);
    idt[Overflow].set_handler_fn(overflow_handler);
    idt[BoundRangeExceeded].set_handler_fn(bound_range_exceeded_handler);
    idt[InvalidOpcode].set_handler_fn(invalid_opcode_handler);
    idt[DeviceNotAvailable].set_handler_fn(device_not_available_handler);
    idt[X87FloatingPoint].set_handler_fn(x87_floating_point_handler);
    idt[SimdFloatingPoint].set_handler_fn(simd_floating_point_handler);
    idt[Virtualization].set_handler_fn(virtualization_handler);

    // TODO: come up with an interface that allows setting both kinds of interrupts

    // [AlignmentCheck]
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    // [MachineCheck]
    idt.machine_check.set_handler_fn(machine_check_handler);
    // [SegmentNotPresent]
    idt.segment_not_present
        .set_handler_fn(segment_not_present_handler);
    // [StackSegmentFault]
    idt.stack_segment_fault
        .set_handler_fn(stack_segment_fault_handler);
    // [GeneralProtectionFault]
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    // [PageFault]
    idt.page_fault.set_handler_fn(page_fault_handler);
    // [SecurityException]
    idt.security_exception
        .set_handler_fn(security_exception_handler);
    // [InvalidTss]
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);

    // [DoubleFault]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    idt[Timer].set_handler_fn(timer);
    idt[Keyboard].set_handler_fn(keyboard);
    idt[MaybeSpurious].set_handler_fn(spurious_interrupt_handler);

    idt.into_inner()
});

pub fn init() {
    unsafe {
        IDT.load();
    }
}
