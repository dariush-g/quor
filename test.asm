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
section .data
fp0: dd 1
section .text
movss xmm0, [fp0]
movss [rbp - 8], xmm0
mov rcx, 0
mov rax, rcx
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
mov rax, qword [rbp - 8]
mov rdx, qword [rax + 8]
mov rdi, rdx
call free
mov r8, qword [rbp - 8]
mov rdi, r8
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
mov r9d, dword [rbp - 12]
mov r11, qword [rbp - 8]
mov r10d, dword [r11 + 0]
cmp r9, r10
setge al
movzx rax, al
cmp rax, 0
je .else0
mov r12, 1
mov rdi, r12
call exit
.else0:
sub rsp, 8
mov r14, qword [rbp - 8]
mov r13, qword [r14 + 8]
mov r15d, dword [rbp - 12]
add r13, r15
mov qword [rbp - 24], r13
mov xmm0, qword [rbp - 24]
mov xmm0, qword [xmm0]
mov rax, xmm0
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
section .data
fmt_int: db "%d",10,0
fmt_char: db "%c",10,0
fmt_str: db "%s",10,0
str_true: db "true",10,0
str_false: db "false",10,0
extern malloc
extern free
extern exit
