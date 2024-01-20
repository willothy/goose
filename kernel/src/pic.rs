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

use core::arch::asm;

fn read_byte(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        asm!("in al, dx", out("al") ret, in("dx") port);
    }
    ret
}

fn write_byte(port: u16, data: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") data);
    }
}

fn init_pit() {
    let data = (1 << 2) | (3 << 4);
    write_byte(0x43, data);
    let divisor = 1193182 / 100;
    write_byte(0x40, (divisor >> 0) as u8);
    write_byte(0x40, (divisor >> 8) as u8);
}

pub mod master {
    pub const COMM: u16 = 0x20;
    pub const DATA: u16 = 0x21;
}

pub mod slave {
    pub const COMM: u16 = 0xA0;
    pub const DATA: u16 = 0xA1;
}

pub mod cmd {
    pub const INIT: u8 = 0x11;
}

fn init_pic() {
    // TODO: do more research and document this
    write_byte(master::COMM, cmd::INIT);
    write_byte(slave::COMM, cmd::INIT);

    write_byte(master::DATA, 0x20 /* 32 */);
    write_byte(slave::DATA, 0x28 /* 40 */);

    write_byte(master::DATA, 0x04);
    write_byte(slave::DATA, 0x02);

    write_byte(master::DATA, 0x01);
    write_byte(slave::DATA, 0x01);

    write_byte(master::DATA, 0b11111110);
    write_byte(slave::DATA, 0b11111111);
}

pub fn init() {
    init_pit();
    init_pic();
}
