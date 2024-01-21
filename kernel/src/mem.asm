global k_memset

k_memset:
	cld
	mov ecx, edx
	mov al, sil
	rep stosb
	ret

global k_memcmp

k_memcmp:
	cld
	xor  eax, eax
	mov  ecx, edx
	repe cmpsb
	setz al
	ret

global k_memcpy
global k_memmove

k_memcpy:
k_memmove:
	;   need to check for overlap
	;   rdi = dest, rsi = src, rdx = count
	cld
	cmp rsi, rdi
	jae .copy
	mov r8, rsi
	add r8, rdx
	cmp r8, rdi
	jbe .copy

.overlap:
	std ; reverse direction
	add rdi, rdx
	add rsi, rdx
	sub rdi, 1
	sub rsi, 1

.copy:
	mov ecx, edx
	rep movsb
	cld ; clear direction in case reversed by overlap
