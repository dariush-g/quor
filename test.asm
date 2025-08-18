extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, rax
mov rax, 60
syscall
global main
main:
push rbp
mov rbp, rsp
sub rsp, 8
mov rcx, 2
mov rdi , rcx
sub rsp, 8
call malloc
add rsp, 8
mov qword [rbp - 8], rax
mov rdx, qword [rbp - 8]
mov r8, 'h'
mov qword [rdx], r8
mov r9, qword [rbp - 8]
mov r10, 1
add r9, r10
mov qword [rbp - 8], r9
mov r11, qword [rbp - 8]
mov r12, 'i'
mov qword [r11], r12
mov r13, qword [rbp - 8]
mov r14, 1
sub r13, r14
mov qword [rbp - 8], r13
mov rcx, qword [rbp - 8]
mov rax, 1
add rcx, rax
mov rdi , rcx
sub rsp, 8
call print_char
add rsp, 8
mov rdx, 0
mov rax, rdx
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
extern malloc
extern free
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
