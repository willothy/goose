#![no_std]
#![no_main]

mod multiboot_header;

global_asm!(include_str!("boot.asm"));
global_asm!(include_str!("load.asm"));

use core::{arch::global_asm, panic::PanicInfo, ptr::addr_of_mut};
use elf::endian::AnyEndian;
use elf::ElfBytes;
use multiboot::information::MemoryManagement;
use multiboot::information::PAddr;

use multiboot::information::{Module, Multiboot};

use common::*;

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

extern "C" {
    // Defined in boot.asm
    fn setup_long_mode();
    // Defined in load.asm
    fn load_kernel(entry: u64);
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
    println(b"long mode enabled");
    load_kernel(entry);

    loop {}
}
