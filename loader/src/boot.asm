/*
credit:
https: // wiki.osdev.org/Bare_Bones#Booting_the_Operating_System
and
https://github.com/sphaerophoria/stream-os/blob/3055554ebcefb1e2fc4776255fc75a24fef00a23/src/boot.s

Comments in this codebase are a mix of copy-paste from the above sources, and my own notes.
*/

.code32

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

	/*
	To set up a stack, we set the esp register to point to the top of the
	stack (as it grows downwards on x86 systems). This is necessarily done
	in assembly as languages such as C cannot function without a stack.
	*/
  mov esp, 0x7c00
  mov ebp, esp

  push ebx
  call loader_main

  cli

  jmp end

.global setup_long_mode
setup_long_mode:
  jmp end

.global load_kernel
load_kernel:
  jmp end

end:
  mov byte ptr [0xb8000], 'E'
  mov byte ptr [0xb8001], 0xf
  hlt
  jmp end

/*
Set the size of the _start symbol to the current location '.' minus its start.
This is useful when debugging or when you implement call tracing.
*/
.size _start, . - _start
