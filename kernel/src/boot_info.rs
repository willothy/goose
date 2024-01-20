use multiboot2::{BootInformation, BootInformationHeader};
use spin::Once;

use crate::println;

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

pub fn dump() {
    let boot_info = get();
    if let Some(loader) = boot_info.boot_loader_name_tag().and_then(|t| t.name().ok()) {
        if loader.as_bytes().len() == 0 {
            println!("Loaded by unknown loader");
        } else {
            println!("Loaded by {}", loader);
        }
    } else {
        println!("No loader name");
    }
    if let Some(cmdline) = boot_info.command_line_tag().and_then(|x| x.cmdline().ok()) {
        if cmdline.as_bytes().len() == 0 {
            println!("Command line unknown");
        } else {
            println!("Command line: {}", cmdline);
        }
    } else {
        println!("No command line");
    }

    if let Some(mem) = boot_info.basic_memory_info_tag() {
        let upper = mem.memory_upper();
        let lower = mem.memory_lower();
        println!("Memory bounds (basic info): 0x{:0X}:0x{:0X}", lower, upper);
    } else {
        println!("No basic memory info");
    }

    if let Some(mem) = boot_info.memory_map_tag() {
        let map = mem.memory_areas();
        println!("Memory map:");
        println!("  Size: 0x{:0X}", mem.entry_size());
        println!("  Version: 0x{:0X}", mem.entry_version());

        for (i, area) in map.iter().enumerate() {
            println!(
                "  Entry {}: 0x{:0X}:0x{:0X}\n  Size: 0x{:0X}",
                i,
                area.start_address(),
                area.end_address(),
                area.size()
            );
        }
    } else {
        println!("No memory map");
    }
}
