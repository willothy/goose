use core::ops::Range;

use multiboot2::{BasicMemoryInfoTag, BootInformation, BootInformationHeader};
use spin::Once;

pub static MULTIBOOT_INFO: Once<BootInformation<'static>> = Once::new();
pub static BOOT_INFO: Once<BootInfo> = Once::new();

#[allow(unused)]
pub struct BootInfo {
    pub info: &'static multiboot2::BootInformation<'static>,
    pub start_addr: usize,
    pub end_addr: usize,
    pub total_size: usize,
    pub loader: &'static str,
    pub cmdline: Option<&'static str>,
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
            .and_then(|cmd| cmd.cmdline().ok())
            .and_then(|cmd| if cmd.len() == 0 { None } else { Some(cmd) });

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

pub fn boot_info() -> &'static BootInfo {
    BOOT_INFO.get().expect("Boot information not initialized")
}
