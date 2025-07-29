global print
print:
push rbp
mov rbp, rsp
mov rax, 5
pop rbp
ret
global _start
_start:
mov rax, 60
xor rdi, rdi
syscall
section .text
global print_int

print_int:
    ; rdi = number to print
    ; preserve caller-saved regs
    push rax
    push rcx
    push rdx
    push rsi
    push rbx

    mov rax, rdi           ; move number to rax for division
    mov rcx, 10
    lea rsi, [rel int_buf + 20] ; start at end of buffer
    mov byte [rsi], 10     ; newline
    dec rsi

    test rax, rax
    jns .convert
    neg rax
    mov bl, '-'            ; remember the sign
    jmp .convert

.convert:
    xor rbx, rbx           ; clear high bits
.loop:
    xor rdx, rdx
    div rcx                ; rax / 10, remainder in rdx
    add dl, '0'
    dec rsi
    mov [rsi], dl
    test rax, rax
    jnz .loop

    cmp bl, '-'
    jne .done
    dec rsi
    mov [rsi], bl

.done:
    mov rdx, int_buf + 21
    sub rdx, rsi           ; rdx = length
    mov rax, 1             ; syscall: write
    mov rdi, 1             ; stdout
    mov rsi, rsi           ; pointer to string
    syscall

    ; restore
    pop rbx
    pop rsi
    pop rdx
    pop rcx
    pop rax
    ret

section .bss
int_buf resb 21  ;
