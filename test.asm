extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, rax
mov rax, 60
syscall
; ----- Layout: string -----
%define string_size 16
%define string.size 0
%define string.data 8

global string.new
string.new:
push rbp
mov rbp, rsp
sub rsp, 16
mov dword [rbp - 4], edi
mov qword [rbp - 12], rsi
mov rdi, string_size
call malloc
mov rcx, rax
mov eax, dword [rbp - 4]
mov dword [rcx + 0], eax
mov rax, qword [rbp - 12]
mov qword [rcx + 8], rax
mov rax, rcx
add rsp, 16
mov rsp, rbp
pop rbp
ret

global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
mov rcx, '2'
mov rdi, rcx
call is_alphabetic
mov rdi, rax
call print_bool
mov rdx, 0
mov rax, rdx
jmp .Lret_main
xor rax, rax
.Lret_main:
mov rsp, rbp
pop rbp
ret
global get_index
get_index:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov dword [rbp - 12], esi
sub rsp, 8
mov r9, qword [rbp - 8]
mov r8, qword [r9 + 8]
mov r10d, dword [rbp - 12]
add r8, r10
mov qword [rbp - 24], r8
mov r11, qword [rbp - 24]
mov r11, qword [r11]
mov rax, r11
jmp .Lret_get_index
.Lret_get_index:
mov rsp, rbp
pop rbp
ret
global is_alphabetic
is_alphabetic:
push rbp
mov rbp, rsp
sub rsp, 16
mov byte [rbp - 1], dil
mov r12b, byte [rbp - 1]
mov r13, 'a'
cmp r12, r13
sete al
movzx rax, al
mov r14b, byte [rbp - 1]
mov rcx, 'b'
cmp r14, rcx
sete al
movzx rax, al
cmp rax, 1
je .or_end_0
cmp rax, 1
je .or_end_0
mov rax, 0
jmp .or_done_1
.or_end_0:
mov rax, 1
.or_done_1:
mov al, byte [rbp - 1]
mov rdx, 'c'
cmp rax, rdx
sete al
movzx rax, al
cmp rax, 1
je .or_end_1
cmp rax, 1
je .or_end_1
mov rax, 0
jmp .or_done_2
.or_end_1:
mov rax, 1
.or_done_2:
mov r9b, byte [rbp - 1]
mov r10, 'd'
cmp r9, r10
sete al
movzx rax, al
cmp rax, 1
je .or_end_2
cmp rax, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov r8b, byte [rbp - 1]
mov r11, 'e'
cmp r8, r11
sete al
movzx rax, al
cmp rax, 1
je .or_end_3
cmp rax, 1
je .or_end_3
mov rax, 0
jmp .or_done_4
.or_end_3:
mov rax, 1
.or_done_4:
mov r12b, byte [rbp - 1]
mov r13, 'f'
cmp r12, r13
sete al
movzx rax, al
cmp rax, 1
je .or_end_4
cmp rax, 1
je .or_end_4
mov rax, 0
jmp .or_done_5
.or_end_4:
mov rax, 1
.or_done_5:
mov r14b, byte [rbp - 1]
mov rcx, 'g'
cmp r14, rcx
sete al
movzx rax, al
cmp rax, 1
je .or_end_5
cmp rax, 1
je .or_end_5
mov rax, 0
jmp .or_done_6
.or_end_5:
mov rax, 1
.or_done_6:
mov al, byte [rbp - 1]
mov rax, 'h'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_6
cmp rax, 1
je .or_end_6
mov rax, 0
jmp .or_done_7
.or_end_6:
mov rax, 1
.or_done_7:
mov al, byte [rbp - 1]
mov rdx, 'i'
cmp rax, rdx
sete al
movzx rax, al
cmp rax, 1
je .or_end_7
cmp rax, 1
je .or_end_7
mov rax, 0
jmp .or_done_8
.or_end_7:
mov rax, 1
.or_done_8:
mov al, byte [rbp - 1]
mov rax, 'j'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_8
cmp rax, 1
je .or_end_8
mov rax, 0
jmp .or_done_9
.or_end_8:
mov rax, 1
.or_done_9:
mov r9b, byte [rbp - 1]
mov r10, 'k'
cmp r9, r10
sete al
movzx rax, al
cmp rax, 1
je .or_end_9
cmp rax, 1
je .or_end_9
mov rax, 0
jmp .or_done_10
.or_end_9:
mov rax, 1
.or_done_10:
mov al, byte [rbp - 1]
mov rax, 'l'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_10
cmp rax, 1
je .or_end_10
mov rax, 0
jmp .or_done_11
.or_end_10:
mov rax, 1
.or_done_11:
mov r8b, byte [rbp - 1]
mov r11, 'm'
cmp r8, r11
sete al
movzx rax, al
cmp rax, 1
je .or_end_11
cmp rax, 1
je .or_end_11
mov rax, 0
jmp .or_done_12
.or_end_11:
mov rax, 1
.or_done_12:
mov al, byte [rbp - 1]
mov rax, 'n'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_12
cmp rax, 1
je .or_end_12
mov rax, 0
jmp .or_done_13
.or_end_12:
mov rax, 1
.or_done_13:
mov r12b, byte [rbp - 1]
mov r13, 'o'
cmp r12, r13
sete al
movzx rax, al
cmp rax, 1
je .or_end_13
cmp rax, 1
je .or_end_13
mov rax, 0
jmp .or_done_14
.or_end_13:
mov rax, 1
.or_done_14:
mov al, byte [rbp - 1]
mov rax, 'p'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_14
cmp rax, 1
je .or_end_14
mov rax, 0
jmp .or_done_15
.or_end_14:
mov rax, 1
.or_done_15:
mov r14b, byte [rbp - 1]
mov rcx, 'q'
cmp r14, rcx
sete al
movzx rax, al
cmp rax, 1
je .or_end_15
cmp rax, 1
je .or_end_15
mov rax, 0
jmp .or_done_16
.or_end_15:
mov rax, 1
.or_done_16:
mov al, byte [rbp - 1]
mov rax, 'r'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_16
cmp rax, 1
je .or_end_16
mov rax, 0
jmp .or_done_17
.or_end_16:
mov rax, 1
.or_done_17:
mov al, byte [rbp - 1]
mov rax, 's'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_17
cmp rax, 1
je .or_end_17
mov rax, 0
jmp .or_done_18
.or_end_17:
mov rax, 1
.or_done_18:
mov al, byte [rbp - 1]
mov rax, 't'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_18
cmp rax, 1
je .or_end_18
mov rax, 0
jmp .or_done_19
.or_end_18:
mov rax, 1
.or_done_19:
mov al, byte [rbp - 1]
mov rdx, 'u'
cmp rax, rdx
sete al
movzx rax, al
cmp rax, 1
je .or_end_19
cmp rax, 1
je .or_end_19
mov rax, 0
jmp .or_done_20
.or_end_19:
mov rax, 1
.or_done_20:
mov al, byte [rbp - 1]
mov rax, 'v'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_20
cmp rax, 1
je .or_end_20
mov rax, 0
jmp .or_done_21
.or_end_20:
mov rax, 1
.or_done_21:
mov al, byte [rbp - 1]
mov rax, 'w'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_21
cmp rax, 1
je .or_end_21
mov rax, 0
jmp .or_done_22
.or_end_21:
mov rax, 1
.or_done_22:
mov al, byte [rbp - 1]
mov rax, 'x'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_22
cmp rax, 1
je .or_end_22
mov rax, 0
jmp .or_done_23
.or_end_22:
mov rax, 1
.or_done_23:
mov r9b, byte [rbp - 1]
mov r10, 'y'
cmp r9, r10
sete al
movzx rax, al
cmp rax, 1
je .or_end_23
cmp rax, 1
je .or_end_23
mov rax, 0
jmp .or_done_24
.or_end_23:
mov rax, 1
.or_done_24:
mov al, byte [rbp - 1]
mov rax, 'z'
cmp rax, rax
sete al
movzx rax, al
cmp rax, 1
je .or_end_24
cmp rax, 1
je .or_end_24
mov rax, 0
jmp .or_done_25
.or_end_24:
mov rax, 1
.or_done_25:
cmp rax, 0
je .else25
mov rax, 1
.else25:
mov rax, 0
jmp .Lret_is_alphabetic
.Lret_is_alphabetic:
mov rsp, rbp
pop rbp
ret
extern printf

global print_int
print_int:
    push rbp
    mov rbp, rsp
    sub rsp, 16         ;
    mov rsi, rdi
    mov rdi, fmt_int
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

; print_bool: rdi = 0 or 1
global print_bool
print_bool:
    push rbp
    mov rbp, rsp
    sub rsp, 16         
    cmp rdi, 0
    mov rdi, str_false
    mov rsi, str_true
    cmovne rdi, rsi
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

; print_char: rdi = char
global print_char
print_char:
    push rbp
    mov rbp, rsp
    sub rsp, 16        
    mov rsi, rdi
    mov rdi, fmt_char
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret

section .data
fmt_int: db "%d",10,0
fmt_char: db "%c",10,0
str_true: db "true",10,0
str_false: db "false",10,0
extern malloc
extern free

