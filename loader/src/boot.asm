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
  mov eax, 0x1
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

  push ebx
  call loader_main

  jmp error /* We should never get here */

/*
Set the size of the _start symbol to the current location '.' minus its start.
This is useful when debugging or when you implement call tracing.
*/
.size _start, . - _start

.global setup_long_mode
setup_long_mode:
  jmp end

.global load_kernel
load_kernel:
  jmp end

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
  jmp end

end:
  hlt
  jmp end

