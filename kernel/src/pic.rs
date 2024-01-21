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

use spin::mutex::Mutex;
use x86_64::instructions::port::Port;

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

    #[inline(always)]
    pub fn command(&mut self, data: u8) {
        unsafe {
            self.comm.write(data);
        }
    }

    #[inline(always)]
    pub fn read(&mut self) -> u8 {
        unsafe { self.data.read() }
    }

    #[inline(always)]
    pub fn write(&mut self, data: u8) {
        unsafe {
            self.data.write(data);
        }
    }

    #[inline(always)]
    pub fn offset(&self) -> u8 {
        self.offset
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn read_cmd(&mut self) -> u8 {
        unsafe { self.comm.read() }
    }
}

pub struct PicPair {
    pub pic_1: Pic,
    pub pic_2: Pic,
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
        let mask1 = self.pic_1.read();
        let mask2 = self.pic_2.read();

        unsafe {
            // Send initialization commands
            self.pic_1.command(INIT_CMD);
            wait_port.write(0);
            self.pic_2.command(INIT_CMD);
            wait_port.write(0);

            // Setup offsets
            self.pic_1.write(self.pic_1.offset());
            wait_port.write(0);
            self.pic_2.write(self.pic_2.offset());
            wait_port.write(0);

            // Setup chaining between PIC1 and PIC2
            self.pic_1.write(0x04);
            wait_port.write(0);
            self.pic_2.write(0x02);
            wait_port.write(0);

            // Set PICs to 8086/88 mode
            self.pic_1.write(MODE_8086);
            wait_port.write(0);
            self.pic_2.write(MODE_8086);
            wait_port.write(0);

            // Restore saved masks
            self.pic_1.write(mask1);
            self.pic_2.write(mask2);
        }
    }

    pub fn end_interrupt(&mut self, id: u8) {
        let one = self.pic_1.offset <= id && id < self.pic_1.offset + 8;
        let two = self.pic_2.offset <= id && id < self.pic_2.offset + 8;
        if one || two {
            if two {
                self.pic_2.command(INTERRUPT_END_CMD);
            }
            self.pic_1.command(INTERRUPT_END_CMD);
        }
    }
}

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = 0x28;

static mut PICS: Mutex<PicPair> = Mutex::new(PicPair::new(
    Pic::new(0x20, 0x21, PIC_1_OFFSET),
    Pic::new(0xa0, 0xa1, PIC_2_OFFSET),
));

pub fn acquire_pics<'a>() -> spin::MutexGuard<'a, PicPair> {
    unsafe { PICS.lock() }
}

pub fn init() {
    acquire_pics().init();
}

pub fn end_interrupt(id: u8) {
    acquire_pics().end_interrupt(id);
}
