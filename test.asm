extern malloc
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
sub rsp, 40
mov rcx, 'h'
mov QWORD [rbp - 40], rcx
mov rdx, 'e'
mov QWORD [rbp - 32], rdx
mov rsi, 'l'
mov QWORD [rbp - 24], rsi
mov rdi, 'l'
mov QWORD [rbp - 16], rdi
mov r10, 'o'
mov QWORD [rbp - 8], r10
sub rsp, 8
mov r11, 0
lea rcx, [rbp + r11*8 - 40]
mov QWORD [rbp - 48], rcx
mov rdx, 0
mov rax, rdx
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
global Example.func
Example.func:
push rbp
mov rbp, rsp
.Lret_Example.func:
mov rsp, rbp
pop rbp
ret
