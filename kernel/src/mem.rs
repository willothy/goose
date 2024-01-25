use core::{mem::MaybeUninit, ptr::addr_of_mut};

use multiboot2::{MemoryArea, MemoryAreaType};

use crate::println;

static mut FREE_REGIONS: [MaybeUninit<MemoryArea>; 128] = [MaybeUninit::zeroed(); 128];
static mut FREE_REGIONS_COUNT: usize = 0;
static mut FREE_BYTES: usize = 0;

pub fn find_available_regions() {
    let boot_info = crate::boot_info::get();
    let regions = boot_info.mem_map;

    let free_regions = &mut unsafe { *addr_of_mut!(FREE_REGIONS) };
    let free_regions_count = &mut unsafe { *addr_of_mut!(FREE_REGIONS_COUNT) };
    let free_bytes = &mut unsafe { *addr_of_mut!(FREE_BYTES) };

    for region in regions {
        match region.typ().into() {
            MemoryAreaType::Available => {
                free_regions[*free_regions_count] = MaybeUninit::new(region.clone());
                *free_regions_count += 1;
                *free_bytes += region.size() as usize;

                // println!(
                //     "Available region: {:#x} - {:#x}",
                //     region.start_address(),
                //     region.end_address(),
                // );
            }
            MemoryAreaType::Reserved => {
                // println!(
                //     "Reserved region: {:#x} - {:#x}",
                //     region.start_address(),
                //     region.end_address()
                // );
            }
            MemoryAreaType::AcpiAvailable => {}
            MemoryAreaType::ReservedHibernate => {}
            MemoryAreaType::Defective => {
                // println!(
                //     "Defective region: {:#x} - {:#x}",
                //     region.start_address(),
                //     region.end_address()
                // );
            }
            MemoryAreaType::Custom(_) => {}
        }
    }

    println!("Found {} free regions.", *free_regions_count);
    let kb = *free_bytes / 1024;
    let mb = kb / 1024;
    let gb = mb / 1024;
    println!(
        "Found {} B / {} KB / {} MB / {} GB free.",
        *free_bytes, kb, mb, gb
    );
}

extern "C" {
    pub fn k_memset(ptr: *mut u8, value: u8, count: usize);

    pub fn k_memcpy(dst: *mut u8, src: *const u8, size: usize);

    pub fn k_memmove(dst: *const u8, src: *const u8, size: usize);

    pub fn k_memcmp(s1: *const u8, s2: *const u8, n: usize) -> i8;
}
