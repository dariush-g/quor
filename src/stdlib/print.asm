extern printf

; print_int: rdi = int
global print_int
print_int:
    mov rsi, rdi          
    mov rdi, fmt_int
    xor rax, rax           
    call printf
    ret

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
