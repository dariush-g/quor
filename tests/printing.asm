extern _print_int
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
mov rcx, 42
mov rdi , rcx
call _print_int
mov rdx, 0
mov rax, rdx
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
