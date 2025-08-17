extern _malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, rax
mov rax, 0x2000001
syscall
; ----- Layout: Point -----
%define Point_size 8
%define Point_x 0
%define Point_y 4

global Point.new
Point.new:
push rbp
mov rbp, rsp
sub rsp, 16
mov dword [rbp - 8], edi
mov dword [rbp - 16], esi
mov rdi, Point_size
call _malloc
mov rcx, rax
mov eax, dword [rbp - 8]
mov dword [rcx + 0], eax
mov eax, dword [rbp - 16]
mov dword [rcx + 4], eax
mov rax, rcx
add rsp, 16
mov rsp, rbp
pop rbp
ret

global main
main:
push rbp
mov rbp, rsp
mov rcx, 0
mov rdx, 0
mov rdi, rcx
mov rsi, rdx
call Point.new
sub rsp, 8
mov qword [rbp - 8], rax
sub rsp, 8
mov r8, qword [rbp - 8]
mov eax, dword [r8 + 0]
mov dword [rbp - 16], eax
mov r9, 0
mov rax, r9
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
