extern malloc
global _start
_start:
call main
mov rdi, rax
mov rax, 60
syscall
; ----- Layout: string -----
%define string_size 16
%define string.size 0
%define string.data 8

global string.new
string.new:
push rbp
mov rbp, rsp
sub rsp, 16
mov dword [rbp - 4], edi
mov qword [rbp - 12], rsi
mov rdi, string_size
call malloc
mov rcx, rax
mov eax, dword [rbp - 4]
mov dword [rcx + 0], eax
mov rax, qword [rbp - 12]
mov qword [rcx + 8], rax
mov rax, rcx
add rsp, 16
mov rsp, rbp
pop rbp
ret

global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rdi, 5
call malloc
mov byte [rax + 0], 'h'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 'l'
mov byte [rax + 3], 'l'
mov byte [rax + 4], 'o'
mov rdi, 5
mov rsi, rax
call string.new
mov r10, rax
mov r11, 3
mov rdi, r10
mov rsi, r11
call get_index
mov qword [rbp - 8], rax
mov r12, 0
mov rax, r12
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
global free_string
free_string:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov r14, qword [rbp - 8]
mov r13, qword [r14 + 8]
mov rdi, r13
call free
mov rcx, qword [rbp - 8]
mov rdi, rcx
call free
.Lret_free_string:
mov rsp, rbp
pop rbp
ret
global get_index
get_index:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov dword [rbp - 12], esi
mov edx, dword [rbp - 12]
mov r9, qword [rbp - 8]
mov r8d, dword [r9 + 0]
cmp rdx, r8
setge al
movzx rax, al
cmp rax, 0
je .else0
mov r10, 1
mov rdi, r10
call exit
.else0:
sub rsp, 8
mov r12, qword [rbp - 8]
mov r11, qword [r12 + 8]
mov r14d, dword [rbp - 12]
add r11, r14
mov qword [rbp - 24], r11
mov r13, qword [rbp - 24]
mov r13, qword [r13]
mov rax, r13
jmp .Lret_get_index
.Lret_get_index:
mov rsp, rbp
pop rbp
ret
extern printf

global print_int
print_int:
    push rbp
    mov rbp, rsp
    sub rsp, 16         ;
    mov rsi, rdi
    mov rdi, fmt_int
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

; print_bool: rdi = 0 or 1
global print_bool
print_bool:
    push rbp
    mov rbp, rsp
    sub rsp, 16         
    cmp rdi, 0
    mov rdi, str_false
    mov rsi, str_true
    cmovne rdi, rsi
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

; print_char: rdi = char
global print_char
print_char:
    push rbp
    mov rbp, rsp
    sub rsp, 16        
    mov rsi, rdi
    mov rdi, fmt_char
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

section .data
fmt_int: db "%d",10,0
fmt_char: db "%c",10,0
str_true: db "true",10,0
str_false: db "false",10,0
extern malloc
extern free
extern exit
