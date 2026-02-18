extern printf


section .rodata
__q_g_0:
    db "%d", 0

section .data
section .bss
section .text
__q_f_print:
push rbp
mov rbp, rsp
sub rsp, 0

call printf

jmp .Lret_print
.Lret_print:
mov rsp, rbp
pop rbp
ret
__q_f_add:
push rbp
mov rbp, rsp
sub rsp, 0
mov rcx, rdx
add rcx, rax
mov rax, rcx
jmp .Lret_add
mov rdi, 1
mov rsi, 2
call __q_f_add

mov rax, rax
mov qword [rbp - 8], rax
lea rax, qword [rel __q_g_0]
mov rdx, rax
lea eax, dword [rbp - 8]
mov rcx, rax
mov rdi, rdx
mov rsi, rcx
call __q_f_print

mov rax, rax
mov rax, 0
jmp .Lret_add
.Lret_add:
mov rsp, rbp
pop rbp
ret
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
mov rdi, 1
mov rsi, 2
call __q_f_add

mov rax, rax
mov qword [rbp - 8], rax
lea rax, qword [rel __q_g_0]
lea eax, dword [rbp - 8]
mov rcx, rax
mov rdi, rax
mov rsi, rcx
call __q_f_print

mov rdx, rax
mov rax, 0
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
