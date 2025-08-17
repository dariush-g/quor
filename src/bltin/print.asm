extern printf

; print_int: rdi = int
global print_int
print_int:
    mov rsi, rdi          
    mov rdi, fmt_int
    xor rax, rax           
    call printf
    ret

; print_int:
;     mov     rcx, 10          ; divisor
;     lea     rsi, [rsp-32]    ; temporary buffer on stack
;     mov     rbx, rsi

; .convert_loop:
;     xor     rdx, rdx
;     div     rcx              ; rax / 10 → quotient in rax, remainder in rdx
;     add     dl, '0'          ; convert remainder to ASCII
;     dec     rsi
;     mov     [rsi], dl
;     test    rax, rax
;     jnz     .convert_loop

;     mov     rax, 1           ; sys_write
;     mov     rdi, 1
;     mov     rdx, rbx
;     sub     rdx, rsi         ; length = buffer_end - current_ptr
;     mov     rsi, rsi
;     syscall
;     ret

; print_bool: rdi = 0 or 1
global print_bool
print_bool:
    cmp rdi, 0
    mov rdi, str_false
    mov rsi, str_true
    cmovne rdi, rsi        
    xor rax, rax
    call printf
    ret

; print_char: rdi = char
global print_char
print_char:
    mov rsi, rdi
    mov rdi, fmt_char
    xor rax, rax
    call printf
    ret

section .data
fmt_int:  db "%d",10,0
fmt_char: db "%c",10,0
str_true: db "true",10,0
str_false: db "false",10,0
