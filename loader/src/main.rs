#![no_std]
#![no_main]

mod multiboot_header;

global_asm!(include_str!("boot.asm"));

use core::{arch::global_asm, panic::PanicInfo, ptr::addr_of_mut};
use elf::endian::AnyEndian;
use elf::ElfBytes;
use multiboot::information::MemoryManagement;
use multiboot::information::PAddr;

use multiboot::information::{Module, Multiboot};

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
#[allow(dead_code)]
const VGA_BUFFER_SIZE: usize = VGA_WIDTH * VGA_HEIGHT * 2; // each cell is 2 bytes

static mut LAST_ROW: usize = 0;
static mut LAST_COL: usize = 0;

fn print(s: &[u8]) {
    let buf = 0xb8000 as *mut u8;
    s.iter().for_each(|&byte| unsafe {
        if LAST_ROW >= VGA_HEIGHT {
            LAST_ROW = 0;
        }
        if LAST_COL >= VGA_WIDTH {
            LAST_COL = 0;
        }
        match byte {
            b'\n' => {
                LAST_ROW += 1;
                LAST_COL = 0;
            }
            b'\r' => {
                LAST_COL = 0;
            }
            _ => {
                let offset = ((80 * LAST_ROW) + LAST_COL) * 2;
                *buf.add(offset) = byte;
                *buf.add(offset + 1) = 0xf;
                LAST_COL += 1;
            }
        }
    });
}

fn int_to_char(n: u8) -> u8 {
    match n {
        0..=9 => n + 48,
        10..=15 => n + 87,
        _ => panic!(),
    }
}

fn newline() {
    unsafe {
        LAST_ROW += 1;
        LAST_COL = 0;
    }
}

fn println(s: &[u8]) {
    print(s);
    newline();
}

#[panic_handler]
unsafe fn panic(info: &PanicInfo) -> ! {
    let s = info.payload().downcast_ref::<&[u8]>();
    print(b"panic: ");
    println(s.unwrap_or(&"".as_bytes()));
    loop {}
}

struct Mem;

impl MemoryManagement for Mem {
    unsafe fn paddr_to_slice(&self, addr: PAddr, length: usize) -> Option<&'static [u8]> {
        let addr = addr as usize;
        let ptr = core::mem::transmute(addr);
        Some(core::slice::from_raw_parts(ptr, length))
    }

    unsafe fn allocate(&mut self, _length: usize) -> Option<(PAddr, &mut [u8])> {
        None
    }

    unsafe fn deallocate(&mut self, addr: PAddr) {
        if addr != 0 {
            panic!("tried to deallocate memory")
        }
    }
}

static mut MEM: Mem = Mem;

fn load_elf_module(start: u64, end: u64) -> u64 {
    let start = start as usize;
    let end = end as usize;
    let size = end - start;
    let ptr = start as *const u8;
    let slice = unsafe { core::slice::from_raw_parts(ptr, size) };

    let Ok(file) = ElfBytes::<AnyEndian>::minimal_parse(slice) else {
        panic!("failed to parse elf");
    };
    println(b"parsing elf...");

    let Some(segments) = file.segments() else {
        panic!("failed to load segment");
    };
    println(b"loading elf...");

    for segment in segments {
        for i in 0..segment.p_filesz {
            unsafe {
                *((segment.p_paddr as usize + i as usize) as *mut u8) =
                    *((start + segment.p_offset as usize + i as usize) as *const u8);
            }
        }
    }

    return file.ehdr.e_entry;
}

#[allow(dead_code)]
fn print_int(mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[9 - i] = int_to_char((n % 10) as u8);
        n /= 10;
        i += 1;
    }
    print(&buf[10 - i..]);
}

fn print_hex(n: u32) {
    let mut buf = [0u8; 8];
    let mut i = 0;
    let mut n = n;
    while n > 0 {
        buf[7 - i] = int_to_char((n % 16) as u8);
        n /= 16;
        i += 1;
    }
    print(b"0x");
    print(&buf[8 - i..]);
}

#[allow(dead_code)]
fn print_hex_64(n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;
    while n > 0 {
        buf[19 - i] = int_to_char((n % 16) as u8);
        n /= 16;
        i += 1;
    }
    print(b"0x");
    print(&buf[20 - i..]);
}

extern "C" {
    fn setup_long_mode();
    fn load_kernel(entry: u64, mboot_ptr: usize);
}

#[no_mangle]
pub unsafe extern "C" fn loader_main(mboot_ptr: usize) -> ! {
    println(b"Initializing\n");

    let mb = Multiboot::from_ptr(mboot_ptr as u64, addr_of_mut!(MEM).as_mut().unwrap())
        .expect("to find multiboot info");

    println(b"Parsing multiboot info\n");

    let module = mb
        .modules()
        .unwrap()
        .into_iter()
        .find(|module| module.string.is_some_and(|name| name == "KERNEL_BIN"));

    let Some(Module { start, end, .. }) = module else {
        panic!("KERNEL_BIN module not found");
    };

    let entry = load_elf_module(start, end);

    println(b"kernel loaded");
    print(b"entry is null: ");
    if entry == 0 {
        println(b"true");
    } else {
        println(b"false");
    }
    print(b"entry: ");
    print_hex_64(entry);
    newline();

    setup_long_mode();
    load_kernel(entry, mboot_ptr);

    loop {}
}
