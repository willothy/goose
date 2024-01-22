STACK_SZ equ 0x4000
MAX_CPUS equ 16

	;       Multiboot2 header
	section .multiboot

mb_start:
	dd 0xe85250d6; magic (multiboot2)
	dd 0; architecture (i386)
	dd mb_end - mb_start; header length
	dd 0x100000000 - (0xe85250d6 + 0 + (mb_end - mb_start)); checksum

	; optional tags

	;  end tag
	dw 0; type
	dw 0; flags
	dd 8; size

mb_end:

	section .data

no_multiboot:
	db "The kernel was not booted by a multiboot-compatible loader.", 0

no_cpuid_err:
	db "CPUID is not supported on this device.", 0

no_long_mode_err:
	db "Long mode is not supported on this device.", 0

failed_to_enter_long_mode:
	db "Failed to enter long mode.", 0

	section .rodata

gdt:
	; temporary gdt for entering long mode
	; gdt is documented in boot64.asm

.data:
	dq 0; null descriptor
	dq (1<<43) | (1<<44) | (1<<47) | (1<<53); code segment

.pointer:
	dw $ - gdt.data - 1
	dq gdt.data

	[BITS   32]
	section .text
	;       The linker script specifies _start as the entry point to the kernel and the
	;       bootloader will jump to this position once the kernel has been loaded. It
	;       doesn't make sense to return from this function as the bootloader is gone.
	global  _start
	extern  long_mode_entry

_start:
	; The bootloader has loaded us into 32-bit protected mode on a x86
	; machine. Interrupts are disabled. Paging is disabled. The processor
	; state is as defined in the multiboot standard. The kernel has full
	; control of the CPU. The kernel can only make use of hardware features
	; and any code it provides as part of itself. There's no printf
	; function, unless the kernel provides its own <stdio.h> header and a
	; printf implementation. There are no security restrictions, no
	; safeguards, no debugging mechanisms, only what the kernel provides
	; itself. It has absolute and complete power over the
	; machine.

	; Clear interrupts
	cli

	;   Save the multiboot info to ensure we don't clobber it during
	;   stack init.
	mov [stack_top], eax
	mov [stack_top-4], ebx

	; OSDev wiki: To set up a stack, we set the esp register to point to the top of the
	; stack (as it grows downwards on x86 systems). This is necessarily done
	; in assembly as languages such as C cannot function without a stack.

	; This stack init code is copied from stream-os

	;   thanks sphaerophoria <3
	mov eax, 1
	cpuid
	shl ebx, 24
	add ebx, 1
	mov eax, STACK_SZ
	mul ebx
	add eax, stack_bottom
	mov esp, eax

	;   Restore the registers that the bootloader saved for us.
	;   we need these for multiboot info.
	mov eax, [stack_top]
	mov ebx, [stack_top - 4]

	;    Check if we were actually loaded by multiboot
	call check_multiboot

	;    Check if CPUID is supported
	call check_cpuid

	;    Check if long mode is supported
	call check_long_mode

	call clear_bootloader_paging
	call setup_page_tables
	call enable_paging

	lgdt [gdt.pointer]

	jmp 8:long_mode_entry

	mov eax, failed_to_enter_long_mode
	jmp error

check_multiboot:
	cmp eax, 0x36d76289
	je  .has_multiboot
	mov eax, no_multiboot
	jmp error

.has_multiboot:
	ret

check_cpuid:
	; We need to check if CPUID is supported before attempting to flip the id bit (21)
	; in the flags register.

	pushfd ; copy flags in eax to stack
	pop    eax

	mov ecx, eax; copy flags to ecx

	xor eax, (1 << 21); flip id bit

	push eax; push new flags to stack
	popfd

	;   copy flags back to eax, with flipped id bit if supported
	pushfd
	pop eax

	;    restore flags from oldd version stored in ecx
	push ecx
	popfd

	;   compare the old and new flags, and error if they are the same
	cmp eax, ecx
	je  .no_cpuid
	ret

.no_cpuid:
	mov eax, no_cpuid_err
	jmp error

check_long_mode:
	;   Check if extended info is supported
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb  .no_long_mode

	;    use extended into to test for long mode
	mov  eax, 0x80000001; extended info
	cpuid
	test edx, (1 << 29); test if LM bit is set
	jz   .no_long_mode
	ret

.no_long_mode:
	mov eax, no_long_mode_err
	jmp error

clear_bootloader_paging:
	;   Clear any paging that the bootloader may have setup.
	mov eax, cr0
	and eax, ~(1 << 31)
	mov cr0, eax

	ret

setup_page_tables:
	;   map first p4 entry to p3 table
	mov eax, p3_table
	or  eax, 0b11; present, writable
	mov [p4_table], eax

	;   map first p3 entry to p2 table
	mov eax, p2_table
	or  eax, 0b11; present, writable
	mov [p3_table], eax

	mov ecx, 0

.map_p2_table:
	mov eax, 0x200000; 2 MiB
	mul ecx
	or  eax, 0b10000011; present, writable, 2 MiB page (huge)
	mov [p2_table + ecx * 8], eax

	inc ecx
	cmp ecx, 512
	jne .map_p2_table

	ret

enable_paging:
	;   Zero out some memory
	;   TODO: why did course say to do this?
	;   This may be redundant bc I have already setup the
	;   stack.
	;   This has to do with PWT I think
	mov edi, 0x80000
	xor eax, eax
	mov ecx, 0x4000                                       // 16384
	rep stosd

	;   TODO: note why we are doing this. I do not know yet.
	;   Course said to do it and that it will be explained later.
	mov dword [0x80000], 0x81007
	mov dword [0x81000], 0b10000111

	;   Load P4 table into CR3
	mov eax, p4_table
	mov cr3, eax

	;   See https:                // wiki.osdev.org/CPU_Registers_x86
	mov eax, cr4
	or  eax, (1 << 5); Enable PAE
	mov cr4, eax

	; Enable page-lebel writethrough (PWT)

	; CR3:
	; The CR3 register is used for holding the base address of the page directory (thanks copilot?)

	;   Bits 3 and 4 are flags:
	;   3: Page-level write-through (PWT)
	;   4: Page-level cache disable (PCD)
	;   PWT and PCD are not used if bit 17 of cr4 (PCIDE) is set. (TODO: what is PCIDE?)
	;   Bits 12-31 (or 63 in long mode) are the page directory base address (PDBR)
	mov eax, 0x80000; 0x80000 = 1 << 19
	mov cr3, eax

	;   set the long mode bit in EFER MSR (model-specific register)
	mov ecx, 0xC0000080
	rdmsr
	or  eax, (1 << 8)
	wrmsr

	;   Enable paging
	mov eax, cr0
	or  eax, (1 << 31)
	mov cr0, eax

	ret

error:
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov ebx, 0; offset
	mov ecx, 0xb8008
	jmp .err_loop_start
	; print null-terminated string in eax to screen

.err_loop_start:
	push eax
	add  eax, ebx
	mov  al, [eax]
	cmp  al, 0
	je   .err_end
	mov  byte [ecx], al
	add  ecx, 1
	mov  byte [ecx], 0xf
	add  ecx, 1
	add  ebx, 1
	pop  eax
	jmp  .err_loop_start

.err_end:
	hlt
	jmp .err_end

section .bss
align   4096

p4_table:
	resb 4096

p3_table:
	resb 4096

p2_table:
	resb 4096

	global stack_top

stack_bottom:
	resb STACK_SZ * MAX_CPUS

stack_top:
