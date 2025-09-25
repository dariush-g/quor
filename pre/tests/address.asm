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
sub rsp, 8
mov rcx, 5
mov QWORD [rbp - 8], rcx
sub rsp, 8
lea rdx, [rbp - 8]
mov QWORD [rbp - 16], rdx
sub rsp, 8
mov rsi, QWORD [rbp - 16]
mov QWORD [rbp - 24], rsi
mov rdi, 0
mov rax, rdi
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
