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

use x86_64::instructions::port::Port;

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

pub mod pic_1 {
    use x86_64::instructions::port::Port;

    pub static mut COMM: Port<u8> = Port::new(0x20);
    pub static mut DATA: Port<u8> = Port::new(0x21);
    pub const OFFSET: u8 = 0x20;
    pub const CASCADE: u8 = 0x04;
}

pub mod pic_2 {
    use x86_64::instructions::port::Port;

    pub static mut COMM: Port<u8> = Port::new(0xA0);
    pub static mut DATA: Port<u8> = Port::new(0xA1);
    pub const OFFSET: u8 = 0x28;
    pub const CASCADE: u8 = 0x02;
}

const INIT_CMD: u8 = 0x11;
const INTERRUPT_END_CMD: u8 = 0x20;
const MODE_8086: u8 = 0x01;

fn init_pic() {
    // TODO: do more research and document this

    // Write garbage data to ports to add some delay for the PICs to initialize.
    // This is sometimes necessary because the PICs are slow to initialize on older hardware.
    // TODO: is this really necessary?
    //
    // source: [pic8295](https://docs.rs/pic8259/latest)
    let mut wait_port: Port<u8> = Port::new(0x80);

    // TODO: do I need to do this?
    let mask1 = unsafe { pic_1::DATA.read() };
    let mask2 = unsafe { pic_2::DATA.read() };

    unsafe {
        // Send initialization commands
        pic_1::COMM.write(INIT_CMD);
        wait_port.write(0);
        pic_2::COMM.write(INIT_CMD);
        wait_port.write(0);

        // Setup offsets
        pic_1::DATA.write(pic_1::OFFSET);
        wait_port.write(0);
        pic_2::DATA.write(pic_2::OFFSET);
        wait_port.write(0);

        // Setup chaining between PIC1 and PIC2
        pic_1::DATA.write(pic_1::CASCADE);
        wait_port.write(0);
        pic_2::DATA.write(pic_2::CASCADE);
        wait_port.write(0);

        // Set PICs to 8086/88 mode
        pic_1::DATA.write(MODE_8086);
        wait_port.write(0);
        pic_2::DATA.write(MODE_8086);
        wait_port.write(0);

        // Restore saved masks
        pic_1::DATA.write(mask1);
        pic_2::DATA.write(mask2);
    }
}

pub fn init() {
    init_pit();
    init_pic();
}

pub fn end_interrupt(id: u8) {
    let one = pic_1::OFFSET <= id && id < pic_1::OFFSET + 8;
    let two = pic_2::OFFSET <= id && id < pic_2::OFFSET + 8;
    if one || two {
        if two {
            unsafe {
                pic_2::COMM.write(INTERRUPT_END_CMD);
            }
        }
        unsafe {
            pic_1::COMM.write(INTERRUPT_END_CMD);
        }
    }
}
