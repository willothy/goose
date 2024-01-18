///! Declare a multiboot header that marks the program as a kernel. These are magic
///! values that are documented in the multiboot standard. The bootloader will
///! search for this signature in the first 8 KiB of the kernel file, aligned at a
///! 32-bit boundary. The signature is in its own section so the header can be
///! forced to be within the first 8 KiB of the kernel file.

/// The multiboot header structure
#[repr(C)]
pub struct MultibootHeader {
    magic: i32,
    flags: i32,
    checksum: i32,
}

/// Align loaded modules on page boundaries
pub const ALIGN: i32 = 1 << 0;
/// Provide memory map
pub const MEMINFO: i32 = 1 << 1;

/// This is the Multiboot 'flag' field
pub const FLAGS: i32 = ALIGN | MEMINFO;
/// 'magic number' lets bootloader find the header
pub const MAGIC: i32 = 0x1BADB002;
/// Checksum of above, to prove we are multiboot
pub const CHECKSUM: i32 = -(MAGIC + FLAGS);

/// NOTE: The multiboot header being declared in Rust *requires* the KEEP() flag in
/// linker script around the multiboot section, otherwise it will be optimized out.
///
/// The #[used] attr may not be required, but it guarantees that the symbol is kept around for
/// compilation. It does not guarantee that the symbol will be kept in the final binary (after
/// linking), which is why we need the KEEP() flag as well. This just guarantees that the symbol
/// won't be optimized out *before* linking.
#[used]
#[link_section = ".multiboot"]
pub static HEADER: MultibootHeader = MultibootHeader {
    magic: MAGIC,
    flags: FLAGS,
    checksum: CHECKSUM,
};
