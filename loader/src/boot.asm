/*
credit:
https: // wiki.osdev.org/Bare_Bones#Booting_the_Operating_System
and
https://github.com/sphaerophoria/stream-os/blob/3055554ebcefb1e2fc4776255fc75a24fef00a23/src/boot.s

Comments in this codebase are a mix of copy-paste from the above sources, and my own notes.
*/

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
  mov dword ptr [0x80000], 0x81007
  mov dword ptr [0x81000], 0b10000111

  /*
  Clear any paging that the bootloader may have setup.
  */
  mov eax, cr0
  and eax, ~(1 << 31)
  mov cr0, eax

  /*
  Setup the GDT
  */
  lgdt [loader_gdt_ptr]

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

  /*
  Enable long mode
  */
  mov ecx, 0xc0000080
  rdmsr
  or eax, 1 << 8 /* Enable long mode */
  wrmsr

  /*
  Enable paging
  */
  mov eax, cr0
  or eax, (1 << 31)
  mov cr0, eax

  ret /* Return to Rust code, which will then call load_kernel */

.section .data
loader_gdt:
  /* Null segment */
  .quad 0x0
  /* Kernel code segment */
  .quad 0x0020980000000000
  /* No need for data segment in loader GDT, because we are in ring 0 */
.equ loader_gdt_len, . - loader_gdt
loader_gdt_ptr:
  .word loader_gdt_len - 1
  .long loader_gdt
