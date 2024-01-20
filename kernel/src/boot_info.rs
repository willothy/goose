use multiboot2::{BootInformation, BootInformationHeader};
use spin::Once;

use common::print as _print;
use common::*;

pub static MULTIBOOT_INFO: Once<BootInformation> = Once::new();

pub fn init(mboot_ptr: usize) -> Result<(), ()> {
    let boot_info = unsafe { BootInformation::load(mboot_ptr as *const BootInformationHeader) };
    let Ok(boot_info) = boot_info else {
        return Err(());
    };
    MULTIBOOT_INFO.call_once(move || boot_info);
    Ok(())
}

pub fn get() -> &'static BootInformation<'static> {
    MULTIBOOT_INFO
        .get()
        .expect("Boot information not initialized")
}

pub fn print() {
    let boot_info = get();
    if let Some(loader) = boot_info.boot_loader_name_tag().and_then(|t| t.name().ok()) {
        _print(b"Loaded by ");
        if loader.as_bytes().len() == 0 {
            println(b"unknown loader");
        } else {
            println(loader.as_bytes());
        }
    } else {
        println(b"No loader name");
    }
    if let Some(cmdline) = boot_info.command_line_tag().and_then(|x| x.cmdline().ok()) {
        _print(b"Command line: ");
        if cmdline.as_bytes().len() == 0 {
            println(b"unknown");
        } else {
            println(cmdline.as_bytes());
        }
    } else {
        println(b"No command line");
    }

    if let Some(mem) = boot_info.basic_memory_info_tag() {
        let upper = mem.memory_upper();
        let lower = mem.memory_lower();
        _print(b"Memory bounds (basic info): ");
        print_hex(lower);
        _print(b" : ");
        print_hex(upper);
        newline();
    } else {
        println(b"No basic memory info");
    }

    if let Some(mem) = boot_info.memory_map_tag() {
        let entry_size = mem.entry_size();
        let entry_ver = mem.entry_version();
        let map = mem.memory_areas();
        println(b"Memory map:");
        _print(b"  Size: ");
        print_hex(entry_size);
        newline();
        _print(b"  Version: ");
        print_hex(entry_ver);
        newline();
        for (i, area) in map.iter().enumerate() {
            _print(b"  Entry ");
            print_int(i as u32);
            _print(b" : ");
            print_hex_64(area.end_address());
            _print(b" : ");
            print_hex_64(area.size());
            newline();
        }
    } else {
        println(b"No memory map");
    }
}
