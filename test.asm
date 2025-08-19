extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, rax
mov rax, 60
syscall
; ----- Layout: Point -----
%define Point_size 8
%define Point_x 0
%define Point_y 4

global Point.new
Point.new:
push rbp
mov rbp, rsp
sub rsp, 8
mov dword [rbp - 8], edi
mov dword [rbp - 8], esi
mov rdi, Point_size
call malloc
mov rcx, rax
mov eax, dword [rbp - 8]
mov dword [rcx + 0], eax
mov eax, dword [rbp - 8]
mov dword [rcx + 4], eax
mov rax, rcx
add rsp, 16
mov rsp, rbp
pop rbp
ret

; ----- Layout: X -----
%define X_size 4
%define X_x 0

global X.new
X.new:
push rbp
mov rbp, rsp
sub rsp, 4
mov dword [rbp - 4], edi
mov rdi, X_size
call malloc
mov rcx, rax
mov eax, dword [rbp - 4]
mov dword [rcx + 0], eax
mov rax, rcx
add rsp, 8
mov rsp, rbp
pop rbp
ret

global main
main:
push rbp
mov rbp, rsp
mov rcx, 5
mov rdi, rcx
call X.new
sub rsp, 8
mov qword [rbp - 8], rax
mov rax, qword [rbp - 8]
mov edx, dword [rax + 0]
mov rdi , rdx
sub rsp, 8
call print_int
add rsp, 8
mov r8, 0
mov rax, r8
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
global x
x:
push rbp
mov rbp, rsp
mov r9, 0
mov rax, r9
jmp .Lret_x
.Lret_x:
mov rsp, rbp
pop rbp
ret
extern printf

; print_int: rdi = int
global print_int
print_int:
    mov rsi, rdi          
    mov rdi, fmt_int
    xor rax, rax           
    call printf
    ret

; print_bool: rdi = 0 or 1
global print_bool
print_bool:
    cmp rdi, 0
    mov rdi, str_false
    mov rsi, str_true
    cmovne rdi, rsi        
    xor rax, rax
    call printf
    ret

; print_char: rdi = char
global print_char
print_char:
    mov rsi, rdi
    mov rdi, fmt_char
    xor rax, rax
    call printf
    ret

section .data
fmt_int:  db "%d",10,0
fmt_char: db "%c",10,0
str_true: db "true",10,0
str_false: db "false",10,0
extern malloc
extern free
