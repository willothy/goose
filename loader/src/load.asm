.set VGA_BASE, 0xb8000
.set VGA_RED, 0x4
.set VGA_WHITE, 0xf

.code64
.global load_kernel
load_kernel:
  pop rbx /* Pop the address of the multiboot info into ebx */
  mov byte ptr [VGA_BASE], 'H'
  mov byte ptr [VGA_BASE + 1], VGA_RED

  /* Call the kernel entry point */
  pop rax
  push rbx /* Push the multiboot info address onto the stack */
  call rax

  jmp end /* We should never get here */
