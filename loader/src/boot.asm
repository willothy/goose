/*
credit:
https: // wiki.osdev.org/Bare_Bones#Booting_the_Operating_System
and
https://github.com/sphaerophoria/stream-os/blob/3055554ebcefb1e2fc4776255fc75a24fef00a23/src/boot.s

Comments in this codebase are a mix of copy-paste from the above sources, and my own notes.
*/
.org 0x7c00

.code32

.set STACK_SIZE, 0x4000 /* 16 KiB */
.set MAX_CPUS, 1
.set VGA_BASE, 0xb8000
.set VGA_RED, 0x4
.set VGA_WHITE, 0xf

/*
The multiboot standard does not define the value of the stack pointer register
(esp) and it is up to the kernel to provide a stack. This allocates room for a
small stack by creating a symbol at the bottom of it, then allocating 16384
bytes for it, and finally creating a symbol at the top. The stack grows
downwards on x86. The stack is in its own section so it can be marked nobits,
which means the kernel file is smaller because it does not contain an
uninitialized stack. The stack on x86 must be 16-byte aligned according to the
System V ABI standard and de-facto extensions. The compiler will assume the
stack is properly aligned and failure to align the stack will result in
undefined behavior.
*/
.section .bss
.align 16
stack_bottom:
  .skip STACK_SIZE * MAX_CPUS
stack_top:
  .skip 4

/*
The linker script specifies _start as the entry point to the kernel and the
bootloader will jump to this position once the kernel has been loaded. It
doesn't make sense to return from this function as the bootloader is gone.
*/
.section .text
.global _start
.type _start, @function
_start:
	/*
	The bootloader has loaded us into 32-bit protected mode on a x86
	machine. Interrupts are disabled. Paging is disabled. The processor
	state is as defined in the multiboot standard. The kernel has full
	control of the CPU. The kernel can only make use of hardware features
	and any code it provides as part of itself. There's no printf
	function, unless the kernel provides its own <stdio.h> header and a
	printf implementation. There are no security restrictions, no
	safeguards, no debugging mechanisms, only what the kernel provides
	itself. It has absolute and complete power over the
	machine.
	*/
  cli

  /*
  Save the multiboot info to ensure we don't clobber it during
  stack init.
  */
  mov [stack_top], eax
  mov [stack_top-4], ebx

	/*
  OSDev wiki: To set up a stack, we set the esp register to point to the top of the
	stack (as it grows downwards on x86 systems). This is necessarily done
	in assembly as languages such as C cannot function without a stack.

  This stack init code is copied from stream-os

  thanks sphaerophoria <3
	*/
  mov eax, 1
  cpuid
  shl ebx, 24
  add ebx, 1
  mov eax, STACK_SIZE
  mul ebx
  add eax, stack_bottom
  mov esp, eax

  /*
  Restore the registers that the bootloader saved for us.
  we need these for multiboot info.
  */
  mov eax, [stack_top]
  mov ebx, [stack_top - 4]

  /*
  Enter the main stage 1 loader.
  The loader is responsible for reading the multiboot info,
  finding the entry point of the kernel in the ELF headers,
  and jumping to it.
  */
  push ebx /* Push the multiboot info address onto the stack */
  call loader_main

  cli
error: /* We should never get here */
  /* lazy way to print "Error" in red */
  mov byte ptr [VGA_BASE], 'E'
  mov byte ptr [VGA_BASE + 1], VGA_RED
  mov byte ptr [VGA_BASE + 2], 'r'
  mov byte ptr [VGA_BASE + 3], VGA_RED
  mov byte ptr [VGA_BASE + 4], 'r'
  mov byte ptr [VGA_BASE + 5], VGA_RED
  mov byte ptr [VGA_BASE + 6], 'o'
  mov byte ptr [VGA_BASE + 7], VGA_RED
  mov byte ptr [VGA_BASE + 8], 'r'
  mov byte ptr [VGA_BASE + 9], VGA_RED
  mov byte ptr [VGA_BASE + 10], 0
  mov byte ptr [VGA_BASE + 11], VGA_RED
end:
  hlt
  jmp end

/*
Set the size of the _start symbol to the current location '.' minus its start.
This is useful when debugging or when you implement call tracing.
*/
.size _start, . - _start

.global setup_long_mode
setup_long_mode:
  cld /* Clear the direction flag */

  /* Zero out the first 16KB memory */
  /*
  TODO: why did course say to do this?
  This may be redundant bc I have already setup the
  stack.
  */
  mov edi, 0x80000
  xor eax, eax
  mov ecx, 0x4000 /* 16384 */
  rep stosd

  /*
  TODO: note why we are doing this. I do not know yet.
  Course said to do it and that it will be explained later.
  */
  mov dword [0x80000], 0x81007
  mov dword [0x81000], 0b10000111

  /*
  Setup the GDT
  */
  lgdt [gdt_ptr]

  /* See https://wiki.osdev.org/CPU_Registers_x86 */
  mov eax, cr4
  or eax, (1<<5) /* Enable PAE (https://wiki.osdev.org/PAE) */
  mov cr4, eax

  /*
  Enable page-lebel writethrough (PWT)

  CR3:
  The CR3 register is used for holding the base address of the page directory (thanks copilot?)

  Bits 3 and 4 are flags:
  3: Page-level write-through (PWT)
  4: Page-level cache disable (PCD)
  PWT and PCD are not used if bit 17 of cr4 (PCIDE) is set. (TODO: what is PCIDE?)
  Bits 12-31 (or 63 in long mode) are the page directory base address (PDBR)
  */
  mov eax, 0x80000 /* 0x80000 = 1 << 19 */
  mov cr3, eax

  ret /* Return to Rust code, which will then call load_kernel */

.section .data
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
