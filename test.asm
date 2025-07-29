global main
main:
push rbp
mov rbp, rsp
mov rax, 1
cmp rax, 1
jne .jmpne0
mov rcx, 0
.jmpne0:
pop rbp
mov rax, 60
xor rdi, rdi
syscall
global example
example:
push rbp
mov rbp, rsp
mov rdx, 5
pop rbp
ret
