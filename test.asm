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
mov rdi, 5
call malloc
mov cl, 'h'
mov byte [rax + 0], cl
mov dl, 'e'
mov byte [rax + 1], dl
mov r8b, 'l'
mov byte [rax + 2], r8b
mov r9b, 'l'
mov byte [rax + 3], r9b
mov r10b, 'o'
mov byte [rax + 4], r10b
mov rdi, 5
mov rsi, rax
call string.new
mov r11, rax
mov r12, 1
mov rdi, r11
mov rsi, r12
call get_index
mov rdi, rax
call print_char
mov r13, 0
mov rax, r13
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
mov rcx, qword [rbp - 8]
mov r14, qword [rcx + 8]
mov edx, dword [rbp - 12]
add r14, rdx
mov qword [rbp - 24], r14
mov r8, qword [rbp - 24]
mov r8, qword [r8]
mov rax, r8
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

