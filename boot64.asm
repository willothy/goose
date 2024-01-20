[BITS 64]

extern kernel_main
extern stack_top

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

    ; mov edi, ebx
    mov edi, [stack_top - 4]
    call kernel_main
    hlt

end:
    hlt
    jmp end
