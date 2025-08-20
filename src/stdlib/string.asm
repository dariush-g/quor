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
.while_start_0:
mov rcx, 1
cmp rcx, 1
jne .while_end_0
sub rsp, 8
mov rdx, 2
mov rdi, rdx
sub rsp, 8
call malloc
add rsp, 8
mov qword [rbp - 8], rax
mov r8, qword [rbp - 8]
mov r9, 'h'
mov byte [r8], r9b
mov r10, qword [rbp - 8]
mov r11d, 1
add r10, r11
mov qword [rbp - 8], r10
mov r12, qword [rbp - 8]
mov r13, 'i'
mov byte [r12], r13b
mov r14, qword [rbp - 8]
mov ecx, 1
sub r14, rcx
mov qword [rbp - 8], r14
mov rdx, 2
mov rax, qword [rbp - 8]
mov rdi, rdx
mov rsi, rax
sub rsp, 8
call string.new
add rsp, 8
sub rsp, 8
mov qword [rbp - 16], rax
sub rsp, 8
lea r8, [rbp - 16]
mov r9, 1
mov rdi, r8
mov rsi, r9
sub rsp, 8
call get_index
add rsp, 8
mov qword [rbp - 24], rax
mov r11b, byte [rbp - 24]
mov rdi, r11
sub rsp, 8
call print_char
add rsp, 8
jmp .while_start_0
.while_end_0:
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
mov r13, qword [rbp - 8]
mov r12, qword [r13 + 8]
mov ecx, dword [rbp - 12]
add r12, rcx
mov qword [rbp - 24], r12
mov r14, qword [rbp - 24]
mov r14, qword [r14]
mov rax, r14
jmp .Lret_get_index
.Lret_get_index:
mov rsp, rbp
pop rbp
ret
global to_arr
to_arr:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
sub rsp, 8
push rdi
mov rdi, 1
add rsp, 8
call malloc
mov rdx, rax
mov rax, rdx
jmp .Lret_to_arr
.Lret_to_arr:
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

