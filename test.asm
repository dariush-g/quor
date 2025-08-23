extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, 10
call print_char
mov rdi, rbx
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
section .data
fp0: dd 1
section .text
movss xmm0, [fp0]
movss [rbp - 8], xmm0
sub rsp, 8
section .data
fp1: dd 2
section .text
movss xmm1, [fp1]
movss [rbp - 16], xmm1
mov ecx, dword [rbp - 8]
mov edx, dword [rbp - 16]
cmp rcx, rdx
setl al
movzx rax, al
mov rdi, rax
call print_bool
mov r8d, dword [rbp - 8]
mov rdi, r8
call print_fp
mov r9, 0
mov rax, r9
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
mov r11, qword [rbp - 8]
mov r10, qword [r11 + 8]
mov rdi, r10
call free
mov r12, qword [rbp - 8]
mov rdi, r12
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
mov r13d, dword [rbp - 12]
mov r15, qword [rbp - 8]
mov r14d, dword [r15 + 0]
cmp r13, r14
setge al
movzx rax, al
cmp rax, 0
je .else0
mov rcx, 1
mov rdi, rcx
call exit
.else0:
sub rsp, 8
mov r11, qword [rbp - 8]
mov rdx, qword [r11 + 8]
mov r10d, dword [rbp - 12]
add rdx, r10
mov qword [rbp - 24], rdx
mov rax, qword [rbp - 24]
mov rax, qword [rax]
jmp .Lret_get_index
.Lret_get_index:
mov rsp, rbp
pop rbp
ret
global concat
concat:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov qword [rbp - 16], rsi
mov rdi, 11
call malloc
mov byte [rax + 0], 'p'
mov byte [rax + 1], 'l'
mov byte [rax + 2], 'a'
mov byte [rax + 3], 'c'
mov byte [rax + 4], 'e'
mov byte [rax + 5], 'h'
mov byte [rax + 6], 'o'
mov byte [rax + 7], 'l'
mov byte [rax + 8], 'd'
mov byte [rax + 9], 'e'
mov byte [rax + 10], 'r'
mov rdi, 11
mov rsi, rax
call string.new
mov rcx, rax
mov rax, rcx
jmp .Lret_concat
.Lret_concat:
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
global print_str
print_str:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov rsi, qword [rdi + 8]    
    mov rdi, fmt_str            
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret
global print_fp
print_fp:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov rsi, rdi
    mov rdi, fmt_float
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret


section .data
fmt_int: db "%d",0
fmt_char: db "%c",0
fmt_str: db "%s",0
fmt_float db "%f",0
str_true: db "true",0
str_false: db "false",0

extern malloc
extern free
extern exit
