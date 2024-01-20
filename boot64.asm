[BITS 64]

extern kernel_main

section .text
global long_mode_entry
long_mode_entry:
    ; zero out segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call kernel_main
    hlt
