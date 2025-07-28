
global print_string

; void print_string(char *str);
; rdi = pointer to null-terminated string
print_string:
    push rbp
    mov rbp, rsp

    ; Calculate string length
    mov rsi, rdi      ; rsi = pointer to string
    xor rcx, rcx      ; counter = 0

.length_loop:
    mov al, byte [rsi + rcx]
    cmp al, 0
    je .length_done
    inc rcx
    jmp .length_loop

.length_done:
    ; rcx = length of string

    ; Prepare syscall parameters
    mov rax, 1        ; sys_write
    mov rdi, 1        ; stdout fd = 1
    mov rsi, rsi      ; pointer to string (already in rsi)
    mov rdx, rcx      ; length
    syscall

    ; Restore stack frame and return
    mov rsp, rbp
    pop rbp
    ret
mov eax, 1
add eax, 1
mov ebx, eax
global _start
_start:
mov rax, 60
xor rdi, rdi
syscall