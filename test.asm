extern _malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, rax
mov rax, 0x2000001
syscall
global main
main:
push rbp
mov rbp, rsp
mov rcx, 0
mov rax, rcx
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
global add
add:
push rbp
mov rbp, rsp
sub rsp, 8
mov rdi, qword [rbp - 8]
sub rsp, 8
mov rsi, qword [rbp - 16]
mov rdx, qword [rbp - 8]
mov rax, qword [rbp - 16]
add rdx, rax
mov rax, rdx
jmp .Lret_add
.Lret_add:
mov rsp, rbp
pop rbp
ret
