//! PIT (Programmable Interval Timer) is used to generate interrupts at a specified frequency.
//! It has three channels (0-2), but we only use channel 0.
//!
//! PIT uses 4 IO ports:
//! - 0x40: Channel 0 data port (read/write)
//! - 0x41: Channel 1 data port (read/write)
//! - 0x42: Channel 2 data port (read/write)
//! - 0x43: Mode/Command register (write only, read is ignored)
//!
//! Mode/Command register:
//! - Bits 6-7: Select channel (0-3)
//!   - 00 (0): Channel 0
//!   - 01 (1): Channel 1
//!   - 10 (2): Channel 2
//! - Bits 4-5: Access mode (0-3)
//!   - 00 (0): Latch count value command
//!   - 01 (1): Access mode: lobyte only;
//!   - 10 (2): Access mode: hibyte only;
//!   - 11 (3): Access mode: lobyte/hibyte
//! - Bits 1-3: Operating mode (0-5)
//!   - 000 (0): Interrupt on terminal count
//!   - 001 (1): Hardware re-triggerable one-shot
//!   - 010 (2): Rate generator
//!   - 011 (3): Square wave generator
//!   - 100 (4): Software triggered strobe
//!   - 101 (5): Hardware triggered strobe
//!   - 110 (6): Rate generator, same as 010
//!   - 111 (7): Square wave generator, same as 011
//! - Bits 0: Binaryh/BCD mode (0 = 16-bit binary, 1 = four-digit BCD)

use spin::rwlock::RwLock;
use x86_64::instructions::port::Port;

fn init_pit() {
    let mut port_43: Port<u8> = Port::new(0x43);
    let mut port_40: Port<u8> = Port::new(0x40);
    let data = (1 << 2) | (3 << 4);
    unsafe {
        port_43.write(data);
    }
    let divisor = 1193182 / 100;
    unsafe {
        port_40.write((divisor & 0xff) as u8);
        port_40.write((divisor >> 8) as u8);
    }
}

pub struct Pic {
    comm: Port<u8>,
    data: Port<u8>,
    offset: u8,
}

impl Pic {
    pub const fn new(comm: u16, data: u16, offset: u8) -> Self {
        Self {
            comm: Port::new(comm),
            data: Port::new(data),
            offset,
        }
    }
}

pub struct PicPair {
    pic_1: Pic,
    pic_2: Pic,
}

const INIT_CMD: u8 = 0x11;
const INTERRUPT_END_CMD: u8 = 0x20;
const MODE_8086: u8 = 0x01;

impl PicPair {
    pub const fn new(pic_1: Pic, pic_2: Pic) -> Self {
        Self { pic_1, pic_2 }
    }

    pub fn init(&mut self) {
        // TODO: do more research and document this

        // Write garbage data to ports to add some delay for the PICs to initialize.
        // This is sometimes necessary because the PICs are slow to initialize on older hardware.
        // TODO: is this really necessary?
        //
        // source: [pic8295](https://docs.rs/pic8259/latest)
        let mut wait_port: Port<u8> = Port::new(0x80);

        // TODO: do I need to do this?
        let mask1 = unsafe { self.pic_1.data.read() };
        let mask2 = unsafe { self.pic_2.data.read() };

        unsafe {
            // Send initialization commands
            self.pic_1.comm.write(INIT_CMD);
            wait_port.write(0);
            self.pic_2.comm.write(INIT_CMD);
            wait_port.write(0);

            // Setup offsets
            self.pic_1.data.write(self.pic_1.offset);
            wait_port.write(0);
            self.pic_2.data.write(self.pic_2.offset);
            wait_port.write(0);

            // Setup chaining between PIC1 and PIC2
            self.pic_1.data.write(0x04);
            wait_port.write(0);
            self.pic_2.data.write(0x02);
            wait_port.write(0);

            // Set PICs to 8086/88 mode
            self.pic_1.data.write(MODE_8086);
            wait_port.write(0);
            self.pic_2.data.write(MODE_8086);
            wait_port.write(0);

            // Restore saved masks
            self.pic_1.data.write(mask1);
            self.pic_2.data.write(mask2);
        }
    }

    pub fn end_interrupt(&mut self, id: u8) {
        let one = self.pic_1.offset <= id && id < self.pic_1.offset + 8;
        let two = self.pic_2.offset <= id && id < self.pic_2.offset + 8;
        if one || two {
            if two {
                unsafe {
                    self.pic_2.comm.write(INTERRUPT_END_CMD);
                }
            }
            unsafe {
                self.pic_1.comm.write(INTERRUPT_END_CMD);
            }
        }
    }
}

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = 0x28;

static mut PICS: RwLock<PicPair> = RwLock::new(PicPair::new(
    Pic::new(0x20, 0x21, PIC_1_OFFSET),
    Pic::new(0xa0, 0xa1, PIC_2_OFFSET),
));

pub fn init() {
    init_pit();
    unsafe {
        PICS.write().init();
    }
}

pub fn end_interrupt(id: u8) {
    unsafe {
        PICS.write().end_interrupt(id);
    }
}
