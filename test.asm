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
mov qword [rbp - 8], rax
mov rdx, 'h'
mov rdi, rdx
call print_char
mov r8, qword [rbp - 8]
mov rdi, r8
call free_string
add rsp, 8
jmp .while_start_0
.while_end_0:
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
sub rsp, 8
mov r14, qword [rbp - 8]
mov r13, qword [r14 + 8]
mov ecx, dword [rbp - 12]
add r13, rcx
mov qword [rbp - 24], r13
mov rdx, qword [rbp - 24]
mov rdx, qword [rdx]
mov rax, rdx
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

