use x86_64::instructions::port::Port;

pub fn init() {
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
