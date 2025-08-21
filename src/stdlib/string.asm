extern malloc
global _start
_start:
call main
mov rbx, rax
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
mov rcx, 2
mov rdi, rcx
sub rsp, 8
call malloc
add rsp, 8
mov qword [rbp - 8], rax
mov rdx, qword [rbp - 8]
mov r8, 'h'
mov byte [rdx], r8b
mov r9, qword [rbp - 8]
mov r10d, 1
add r9, r10
mov qword [rbp - 8], r9
mov r11, qword [rbp - 8]
mov r12, 'i'
mov byte [r11], r12b
mov r13, qword [rbp - 8]
mov r14d, 1
sub r13, r14
mov qword [rbp - 8], r13
mov rcx, 2
mov rax, qword [rbp - 8]
mov rdi, rcx
mov rsi, rax
sub rsp, 8
call string.new
add rsp, 8
sub rsp, 8
mov qword [rbp - 16], rax
mov rdx, qword [rbp - 16]
mov r8, 1
mov rdi, rdx
mov rsi, r8
call get_index
mov rdi, rax
call print_char
mov r10, 0
mov rax, r10
jmp .Lret_main
xor rax, rax
.Lret_main:
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
sub rsp, 8
mov r11, qword [rbp - 8]
mov r9, qword [r11 + 8]
mov r12d, dword [rbp - 12]
add r9, r12
mov qword [rbp - 24], r9
mov r14, qword [rbp - 24]
mov r14, qword [r14]
mov rax, r14
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

