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
mov rax, 1
mov r8, 1
mov r9, 1
mov rdi, rcx
mov rsi, rdx
mov rdx, rax
mov rcx, r8
mov r8, r9
call Example_new
sub rsp, 8
mov QWORD [rbp - 8], rax
mov r10, 0
mov rax, r10
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
; ----- Layout: Example -----
%define Example_size 20
%define Example_x 0
%define Example_y 4
%define Example_z 8
%define Example_n 12
%define Example_w 16

global Example_new
Example_new:
push rbp
mov rbp, rsp
sub rsp, 8
push rdi
push rsi
push rdx
push rcx
push r8
mov rdi, Example_size
call _malloc
add rsp, 8
mov rcx, rax
mov eax, dword [rbp - 8]
mov dword [rcx + 0], eax
mov al, byte [rbp - 16]
mov byte [rcx + 4], al
mov eax, dword [rbp - 24]
mov dword [rcx + 8], eax
mov eax, dword [rbp - 32]
mov dword [rcx + 12], eax
mov eax, dword [rbp - 40]
mov dword [rcx + 16], eax
mov rax, rcx
add rsp, 40
mov rsp, rbp
pop rbp
ret

