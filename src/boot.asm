/*
credit:
https: // wiki.osdev.org/Bare_Bones#Booting_the_Operating_System
and
https://github.com/sphaerophoria/stream-os/blob/3055554ebcefb1e2fc4776255fc75a24fef00a23/src/boot.s

Comments in this codebase are a mix of copy-paste from the above sources, and my own notes.
*/

.code32
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
 .skip 16384 /* 16 KiB */
stack_top:

/*
The linker script specifies _start as the entry point to the kernel and the
bootloader will jump to this position once the kernel has been loaded. It
doesn't make sense to return from this function as the bootloader is gone.
*/
.section .text
.global _start
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
	To set up a stack, we set the esp register to point to the top of the
	stack (as it grows downwards on x86 systems). This is necessarily done
	in assembly as languages such as C cannot function without a stack.
	*/
	mov esp, stack_top
  mov ebp, stack_top

	/*
	This is a good place to initialize crucial processor state before the
	high-level kernel is entered. It's best to minimize the early
	environment where crucial features are offline. Note that the
	processor is not fully initialized yet: Features such as floating
	point instructions and instruction set extensions are not initialized
	yet. The GDT should be loaded here. Paging should be enabled here.
	C++ features such as global constructors and exceptions will require
	runtime support to work as well.
	*/
  cld
  mov edi, 0x80000
  xor eax, eax
  mov ecx, 0x1000/4
  rep stosd

  /*
    TODO: what are the numbers???
  */
  mov dword ptr [0x80000], 0x81007
  mov dword ptr [0x81000], 0b10000111

  lgdt [gdt_32_ptr]

  mov eax, cr4
  or eax, (1 << 5)
  mov cr4, eax

  mov eax, 0x80000
  mov cr3, eax

  mov ecx, 0xc0000080
  rdmsr
  or eax, (1 << 8)
  wrmsr

  /*
  Enable paging
  */
  mov eax, cr0
  or eax, (1 << 31)
  mov cr0, eax

  jmp long_mode_entry

.code64
long_mode_entry:
  mov rsp, 0x7c00
  mov rbp, 0x7c00
  /*
  mov rsp, stack_top
  */

  cld

  mov rdi, 0x200000
  mov rsi, 0x10000
  mov rcx, 51200/8
  rep movsq

  lgdt [gdt_64_ptr]

  /*mosv rdi, 0x200000*/

	/*
	Enter the high-level kernel. The ABI requires the stack is 16-byte
	aligned at the time of the call instruction (which afterwards pushes
	the return pointer of size 4 bytes). The stack was originally 16-byte
	aligned above and we've pushed a multiple of 16 bytes to the
	stack since (pushed 0 bytes so far), so the alignment has thus been
	preserved and the call is well defined.
	*/
	call kernel_main

	/*
	If the system has nothing more to do, put the computer into an
	infinite loop. To do that:
	1) Disable interrupts with cli (clear interrupt enable in eflags).
	   They are already disabled by the bootloader, so this is not needed.
	   Mind that you might later enable interrupts and return from
	   kernel_main (which is sort of nonsensical to do).
	2) Wait for the next interrupt to arrive with hlt (halt instruction).
	   Since they are disabled, this will lock up the computer.
	3) Jump to the hlt instruction if it ever wakes up due to a
	   non-maskable interrupt occurring or due to system management mode.
	*/
	cli

  jmp end

end:	hlt
	jmp end


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
  .quad 0x0
  .quad 0x0020980000000000
.equ gdt_64_len, . - gdt
gdt_32_ptr:
  .word gdt_64_len - 1
  .long gdt
gdt_64_ptr:
  .word gdt_64_len - 1
  .quad gdt

/*
Set the size of the _start symbol to the current location '.' minus its start.
This is useful when debugging or when you implement call tracing.
*/
.size _start, . - _start
