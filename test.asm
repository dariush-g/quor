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
sub rsp, 8
mov rcx, 0
mov rax, rcx
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
push rdx
push rsi
mov rdi, Example_size
call _malloc
mov eax, dword [rbp - 8]
mov dword [rax + 0], eax
mov al, byte [rbp - 16]
mov byte [rax + 4], al
add rsp, 8
add rsp, 8
mov rsp, rbp
pop rbp
ret

