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
mov rcx, 5
mov rdx, 'h'
mov rdi, rcx
mov rsi, rdx
call Example_new
sub rsp, 8
mov QWORD [rbp - 8], rax
mov rax, 0
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
; ----- Layout: Example -----
%define Example_size 8
%define Example_x 0
%define Example_y 4

global Example_new
Example_new:
push rbp
mov rbp, rsp
mov rdi, Example_size
call _malloc
mov rcx, rax
mov eax, dword [rbp - 8]
mov dword [rcx + 0], eax
mov al, byte [rbp - 16]
mov byte [rcx + 4], al
mov rax, rcx
add rsp, 16
mov rsp, rbp
pop rbp
ret

