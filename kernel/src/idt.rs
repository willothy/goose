use core::{arch::asm, ptr::addr_of};

/// P: Present
/// DPL: Descriptor Privilege Level
/// Type: Type
///
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// | P | DPL           | Type      |
#[derive(Clone, Copy)]
pub struct PDplAndType(u8);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GateType {
    Interrupt = 0xE,
    Trap = 0xF,
}

impl PDplAndType {
    pub const fn new(p: bool, dpl: u8, ty: GateType) -> Self {
        Self((p as u8) << 7 | (dpl & 0b11) << 5 | (ty as u8 & 0b1111))
    }

    #[inline(always)]
    pub fn is_present(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    #[inline(always)]
    pub fn get_dpl(&self) -> u8 {
        (self.0 & 0b0110_0000) >> 5
    }

    #[inline(always)]
    pub fn get_type(&self) -> GateType {
        match self.0 & 0b0000_1111 {
            0b1110 => GateType::Interrupt,
            0b1111 => GateType::Trap,
            _ => unreachable!("Invalid gate type"),
        }
    }
}

/// Reserved space, and offset into IST (interrupt stack table).
///
/// | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |  Reserved             | IST   |
#[derive(Clone, Copy)]
pub struct ReservedAndIst(u8);

impl ReservedAndIst {
    pub const fn new(ist: u8) -> Self {
        Self(ist)
    }

    #[inline(always)]
    pub fn get_ist(&self) -> u8 {
        self.0 & 0b11
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    segment_selector: u16,
    resv_ist: ReservedAndIst,
    p_dpl_type: PDplAndType,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub fn new(
        handler: extern "x86-interrupt" fn() -> !,
        segment_selector: u16,
        present: bool,
        dpl: u8,
        ty: GateType,
        ist: u8,
    ) -> Self {
        let offset: usize = unsafe { core::mem::transmute(handler) };
        Self {
            reserved: 0,
            p_dpl_type: PDplAndType::new(present, dpl, ty),
            resv_ist: ReservedAndIst::new(ist),
            segment_selector,
            offset_low: (offset & 0xFFFF) as u16,
            offset_mid: ((offset >> 16) & 0xFFFF) as u16,
            offset_high: (offset >> 32) as u32,
        }
    }

    #[inline(always)]
    pub fn set_handler(&mut self, handler: extern "C" fn()) {
        let offset: u64 = unsafe { core::mem::transmute(handler) };
        self.offset_low = (offset & 0xFFFF) as u16;
        self.offset_mid = ((offset >> 16) & 0xFFFF) as u16;
        self.offset_high = (offset >> 32) as u32;
    }

    #[inline(always)]
    pub fn set_present(&mut self, present: bool) {
        self.p_dpl_type.0 = (self.p_dpl_type.0 & 0b0111_1111) | ((present as u8) << 7);
    }

    #[inline(always)]
    pub fn present(&self) -> bool {
        self.p_dpl_type.is_present()
    }

    #[inline(always)]
    pub fn set_dpl(&mut self, dpl: u8) {
        self.p_dpl_type.0 = (self.p_dpl_type.0 & 0b1001_1111) | ((dpl & 0b11) << 5);
    }

    #[inline(always)]
    pub fn dpl(&self) -> u8 {
        self.p_dpl_type.get_dpl()
    }

    #[inline(always)]
    pub fn set_type(&mut self, ty: GateType) {
        self.p_dpl_type.0 = (self.p_dpl_type.0 & 0b1111_0000) | (ty as u8 & 0b1111);
    }

    #[inline(always)]
    pub fn ty(&self) -> GateType {
        self.p_dpl_type.get_type()
    }

    #[inline(always)]
    pub fn set_ist(&mut self, ist: u8) {
        self.resv_ist.0 = (self.resv_ist.0 & 0b1111_1100) | (ist & 0b11);
    }

    #[inline(always)]
    pub fn ist(&self) -> u8 {
        self.resv_ist.get_ist()
    }

    #[inline(always)]
    pub fn set_segment_selector(&mut self, segment_selector: u16) {
        self.segment_selector = segment_selector;
    }

    #[inline(always)]
    pub fn segment_selector(&self) -> u16 {
        self.segment_selector
    }
}

pub struct Idt {
    entries: [IdtEntry; 256],
}

#[repr(C, packed)]
pub struct IdtPtr {
    limit: u16,
    base: u64,
}

static mut IDT: Idt = Idt {
    entries: [IdtEntry {
        reserved: 0,
        offset_high: 0,
        offset_mid: 0,
        p_dpl_type: PDplAndType::new(true, 0, GateType::Interrupt),
        resv_ist: ReservedAndIst::new(0),
        segment_selector: 0x8,
        offset_low: 0,
    }; 256],
};
static mut IDT_PTR: IdtPtr = IdtPtr {
    limit: core::mem::size_of::<Idt>() as u16 - 1,
    base: 0,
};

extern "C" fn default_handler() {
    unsafe {
        asm!("hlt");
    }
}

pub fn init() {
    unsafe {
        IDT_PTR.base = core::mem::transmute(addr_of!(IDT));
    }

    for i in 0..32 {
        unsafe {
            IDT.entries[i].set_handler(default_handler);
        }
    }
}

pub fn load() {
    unsafe {
        asm!("lidt [{idt_ptr}]", idt_ptr = in(reg) addr_of!(IDT_PTR));
    }
}

pub fn set_entry(index: usize, entry: IdtEntry) -> Result<(), ()> {
    match index {
        ..=255 => unsafe {
            IDT.entries[index] = entry;
            Ok(())
        },
        256.. => Err(()),
    }
}

pub fn get_entry<'a>(index: usize) -> Option<&'a IdtEntry> {
    if index > 255 {
        return None;
    }
    Some(unsafe { &IDT.entries[index] })
}

/// # Safety
/// The caller must ensure that the index is valid (<=255), but if the index is valid, then
/// all accesses are safe and valid.
pub fn get_entry_mut<'a>(index: usize) -> &'a mut IdtEntry {
    unsafe { &mut IDT.entries[index] }
}

pub unsafe fn enable_interrupts() {
    asm!("sti");
}

pub unsafe fn disable_interrupts() {
    asm!("cli");
}
