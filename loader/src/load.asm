.set VGA_BASE, 0xb8000
.set VGA_RED, 0x4
.set VGA_WHITE, 0xf

.global load_kernel
load_kernel:
  push ebp
  mov ebp, esp

  mov esi, [ebp + 8] /* Get kernel entry point */
  mov [kernel_ptr], esi

  jmp load_kernel_long

.code64
load_kernel_long:
  mov byte ptr [VGA_BASE], 'H'
  mov byte ptr [VGA_BASE + 1], VGA_RED

  mov rsp, 0x7c00
  mov rbp, rsp

  cld

  mov rsi, [kernel_ptr] /* Get the kernel entry point */
  shl rsi, 32 /* Shift it left 32 bits */

  /* Call the kernel entry point */
  jmp rsi

  mov byte ptr [VGA_BASE], 'E'
  mov byte ptr [VGA_BASE + 1], VGA_RED

  jmp end /* We should never get here */

.section .data
kernel_ptr:
  .long 0x0

/*
  source: https://wiki.osdev.org/Global_Descriptor_Table

  Global Descriptor Table (GDT)

  The GDT is a table of 64-bit entries, each describing a segment of memory.

  Each entry has the following format:
  ┌─────────┬────────┬────────┬─────────────┬──────────────┐
  │ 63   56 │ 55  52 │ 51  48 │ 47       40 │ 39        32 │
  │ Base    │ Flags  │ Limit  │ Access byte │ Base         │
  ├─────────┴────────┴────────┼─────────────┴──────────────┤
  │ 31                     16 │ 15                       0 │
  │ Base                      │ Limit                      │
  └───────────────────────────┴────────────────────────────┘

  Base:

  A 32-bit value containing the linear address where the segment begins.

  Limit:

  A 20-bit value, tells the maximum addressable unit, either in 1 byte units, or in 4KiB pages. Hence, if you choose page
  granularity and set the Limit value to 0xFFFFF the segment will span the full 4 GiB address space in 32-bit mode.

  NOTE: In 64-bit mode, the Base and Limit values are ignored, each descriptor covers
  the entire linear address space regardless of what they are set to."

  Access byte:

  P: Present bit.
      Must be 1 for all valid selectors.
  DPL: Descriptor Privilege Level.
      0 for kernel, 3 for userspace.
  S: Descriptor type bit.
      If clear the desciptor defines a system segment (e.g. TSS).
      If set, the descriptor defines a code or data segment.
  E: Executable bit
      If clear the segment is a data segment.
      If set the segment is a code segment.
  DC: Direction/Conforming bit.
    Direction bit for data segments.
      If clear, the segment grows up. If set, the segment grows down.
    Conforming bit for code segments.
      If set, the segment can be executed from an equal or lower privilege level.
      DPL represents the highest privilege level that is allowed to execute the segment.
  RW: Readable bit/Writable bit.
    Readable bit for code segments.
      If clear, code in this segment can not be read from.
      If set, code in this segment can be read from.
      Code segments are never writeable.
    Writable bit for data segments.
      If clear, data in this segment can not be written to.
      If set, data in this segment can be written to.
  A: Accessed bit.
      Just set to 0. The CPU sets this to 1 when the segment is accessed.

  Flags:

  G: Granularity flag.
      Indicates the size the Limit value is scaled by.
      If clear, the limit is in 1 B blocks (byte granularity).
      If set, the limit is in 4 KiB blocks (page granularity).
  DB: Size flag.
      If clear, the selector defines a 16 bit protected mode segment.
      If set, the selector defines a 32 bit protected mode segment.

      A GDT can have both 16 bit and 32 bit selectors at once. (TODO: figure out what this means)
  L: Long mode flag.
      If set, the selector defines a 64 bit code segment.
      For any other descriptor (code segment or otherwise), this bit must be 0.

      NOTE: When set, DB should always be clear. Essentially, this is mutually exclusive with DB
      because it's an extension from the original 32 bit architecture.

  Attributes of code segment entry:
  D L    P DPL 1 1 C
  0 1    1 00      0
*/
gdt:
  /* Null segment */
  .quad 0x0
  /* Kernel code segment */
  .quad 0x0020980000000000
  /* No need for data segment in loader GDT, because we are in ring 0 */
.equ gdt_len, . - gdt
gdt_ptr:
  .word gdt_len - 1
  .long gdt
