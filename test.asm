global _start
_start:
push rbp
mov rbp, rsp
pop rbp
mov rax, 60
xor rdi, rdi
syscall
global example
example:
push rbp
mov rbp, rsp
mov rax, 5
pop rbp
ret
