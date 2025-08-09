extern _print_bool
extern _print_int
global _start
_start:
push rbp
mov rbp, rsp
sub rsp, 8
mov rcx, 1
mov QWORD [rbp - 8], rcx
mov rdx, QWORD [rbp - 8]
cmp rdx, 0
je .else0
sub rsp, 8
mov rsi, 0
mov QWORD [rbp - 16], rsi
mov rdi, QWORD [rbp - 8]
mov rdi , rdi
call _print_bool
mov r10, QWORD [rbp - 16]
mov rdi , r10
call _print_int
.else0:
call example
mov rsp, rbp
pop rbp
mov rax, 0x2000001
xor rdi, rdi
syscall
global example
example:
push rbp
mov rbp, rsp
sub rsp, 8
mov r11, 5
mov QWORD [rbp - 8], r11
sub rsp, 8
lea rcx, [rbp - 8]
mov QWORD [rbp - 16], rcx
mov rdx, QWORD [rbp - 16]
mov rsi, 100
mov QWORD [rdx], rsi
mov rdi, QWORD [rbp - 8]
mov rdi , rdi
call _print_int
mov rsp, rbp
pop rbp
ret
