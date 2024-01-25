use core::ops::Range;

use multiboot2::{BasicMemoryInfoTag, BootInformation, BootInformationHeader};
use spin::Once;

use crate::println;

pub static MULTIBOOT_INFO: Once<BootInformation<'static>> = Once::new();
pub static BOOT_INFO: Once<BootInfo> = Once::new();

pub struct BootInfo {
    pub info: &'static multiboot2::BootInformation<'static>,
    pub start_addr: usize,
    pub end_addr: usize,
    pub total_size: usize,
    pub loader: &'static str,
    pub cmdline: &'static str,
    pub mem_bounds: Range<u32>,
    pub mem_map: &'static [multiboot2::MemoryArea],
}

pub fn init(mboot_ptr: usize) -> Result<(), ()> {
    let boot_info = unsafe { BootInformation::load(mboot_ptr as *const BootInformationHeader) };
    let Ok(boot_info) = boot_info else {
        return Err(());
    };
    MULTIBOOT_INFO.call_once(move || boot_info);
    BOOT_INFO.call_once(move || {
        let boot_info = MULTIBOOT_INFO.get().unwrap();

        let mem_map = boot_info.memory_map_tag().unwrap().memory_areas();

        let basic_map = boot_info.basic_memory_info_tag().unwrap() as *const BasicMemoryInfoTag;
        let basic_map = unsafe { basic_map.as_ref().unwrap() };
        let bounds = basic_map.memory_lower()..basic_map.memory_upper();

        let start_addr = boot_info.start_address();
        let end_addr = boot_info.end_address();
        let total_size = boot_info.total_size();

        let loader = unsafe {
            boot_info
                .boot_loader_name_tag()
                .map(|name| name.name().unwrap_or("unknown"))
                .map(|s| s as *const str)
                .unwrap_or("unknown" as *const str)
                .as_ref()
                .unwrap()
        };

        let cmdline = boot_info
            .command_line_tag()
            .map(|cmd| {
                cmd.cmdline()
                    .ok()
                    .and_then(|x| if x.len() == 0 { None } else { Some(x) })
                    .unwrap_or("unknown")
            })
            .unwrap_or("unknown");

        BootInfo {
            info: boot_info,
            start_addr,
            end_addr,
            total_size,
            loader,
            cmdline,
            mem_map,
            mem_bounds: bounds,
        }
    });
    Ok(())
}

#[allow(dead_code)]
pub fn get() -> &'static BootInfo {
    BOOT_INFO.get().expect("Boot information not initialized")
}

#[allow(dead_code)]
pub fn dump() {
    let info: &BootInfo = get();

    println!("Loaded by {}", info.loader);
    println!("Command line: {}", info.cmdline);
    println!("Start address: 0x{:0X}", info.start_addr);
    println!("End address: 0x{:0X}", info.end_addr);
    println!("Boot info size: 0x{:0X}", info.total_size);

    println!(
        "Memory bounds (basic info): 0x{:0X}:0x{:0X}",
        info.mem_bounds.start, info.mem_bounds.end
    );

    let map = info.mem_map;
    println!("Memory map:");
    for (i, area) in map.iter().enumerate() {
        println!(
            "  Entry {}: 0x{:0X}:0x{:0X}\n  Size: 0x{:0X}",
            i,
            area.start_address(),
            area.end_address(),
            area.size()
        );
    }

    // println!("Tags: {}", info.info.total_size());
    // info.info.total_size()

    // let frame_buffer = boot_info.framebuffer_tag();
    // let efi_bs_not_exited = boot_info.efi_bs_not_exited_tag();
    // let efi_ih32 = boot_info.efi_ih32_tag();
    // let efi_ih64 = boot_info.efi_ih64_tag();
    // let efi_mem_map = boot_info.efi_memory_map_tag();
    // let efi_sdt32 = boot_info.efi_sdt32_tag();
    // let efi_sdt64 = boot_info.efi_sdt64_tag();
    // let elf_sections = boot_info.elf_sections();
    // let modules = boot_info.module_tags();
    // let rdsp_v1 = boot_info.rsdp_v1_tag();
    // let rdsp_v2 = boot_info.rsdp_v2_tag();
    // let smbios = boot_info.smbios_tag();
    // let vbe_info = boot_info.vbe_info_tag();
}
