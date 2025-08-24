extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, 10
call print_char
mov rdi, rbx
mov rax, 60
syscall
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rdi, 7
call malloc
mov byte [rax + 0], 'h'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 'l'
mov byte [rax + 3], 'l'
mov byte [rax + 4], 'o'
mov byte [rax + 5], ' '
mov byte [rax + 6], 0
mov rbx, rax
mov qword [rbp - 8], rbx
sub rsp, 8
mov rdi, 6
call malloc
mov byte [rax + 0], 'w'
mov byte [rax + 1], 'o'
mov byte [rax + 2], 'r'
mov byte [rax + 3], 'l'
mov byte [rax + 4], 'd'
mov byte [rax + 5], 0
mov r12, rax
mov qword [rbp - 16], r12
sub rsp, 8
mov rcx, qword [rbp - 8]
mov rdx, qword [rbp - 16]
mov rdi, rcx
mov rsi, rdx
call concat
mov qword [rbp - 24], rax
mov r8, qword [rbp - 8]
mov rdi, r8
call free
mov r9, qword [rbp - 16]
mov rdi, r9
call free
mov r10, qword [rbp - 24]
mov rdi, r10
call print_str
mov r11, 0
mov rax, r11
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
global get_char_at
get_char_at:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov dword [rbp - 12], esi
sub rsp, 8
mov r13, qword [rbp - 8]
mov rdi, r13
call strlen
mov qword [rbp - 24], rax
mov r14d, dword [rbp - 12]
mov r15d, dword [rbp - 24]
cmp r14, r15
setge al
movzx rax, al
cmp rax, 0
je .else0
mov rbx, 1
mov rdi, rbx
call exit
.else0:
sub rsp, 8
mov r12, qword [rbp - 8]
mov rcx, qword [rbp - 8]
mov edx, dword [rbp - 12]
add rcx, rdx
mov qword [rbp - 32], rcx
mov r8, qword [rbp - 32]
sub rsp, 8
mov qword [rbp - 40], r8
mov r9, qword [rbp - 32]
mov rdi, r9
call free
mov r10b, byte [rbp - 40]
mov rax, r10
jmp .Lret_get_char_at
.Lret_get_char_at:
mov rsp, rbp
pop rbp
ret
global concat
concat:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov qword [rbp - 16], rsi
sub rsp, 8
mov r11, qword [rbp - 8]
mov rdi, r11
call strlen
mov qword [rbp - 24], rax
sub rsp, 8
mov r13, qword [rbp - 16]
mov rdi, r13
call strlen
mov qword [rbp - 32], rax
sub rsp, 8
sub rsp, 8
mov r14, qword [rbp - 40]
mov qword [rbp - 48], r14
sub rsp, 8
mov r15, 0
mov qword [rbp - 56], r15
.while_start_1:
mov ecx, dword [rbp - 56]
mov r8d, dword [rbp - 24]
cmp rcx, r8
setl al
movzx rax, al
cmp rax, 1
jne .while_end_1
sub rsp, 8
mov r9, qword [rbp - 8]
mov r11d, dword [rbp - 56]
mov rdi, r9
mov rsi, r11
call get_char_at
mov qword [rbp - 64], rax
mov rax, qword [rbp - 40]
mov r15b, byte [rbp - 64]
mov byte [rax], r15b
mov rcx, qword [rbp - 40]
mov r8d, 1
add rcx, r8
mov qword [rbp - 40], rcx
mov eax, dword [rbp - 56]
mov r9, 1
add rax, r9
mov qword [rbp - 56], rax
add rsp, 64
jmp .while_start_1
.while_end_1:
sub rsp, 8
mov rax, 0
mov qword [rbp - 72], rax
.while_start_2:
mov eax, dword [rbp - 72]
mov r15d, dword [rbp - 32]
cmp rax, r15
setl al
movzx rax, al
cmp rax, 1
jne .while_end_2
mov r8, qword [rbp - 16]
mov ecx, dword [rbp - 56]
mov rdi, r8
mov rsi, rcx
call get_char_at
mov qword [rbp - 64], rax
mov r9, qword [rbp - 40]
mov al, byte [rbp - 64]
mov byte [r9], al
mov r15, qword [rbp - 40]
mov r8d, 1
add r15, r8
mov qword [rbp - 40], r15
mov ecx, dword [rbp - 72]
mov rax, 1
add rcx, rax
mov qword [rbp - 72], rcx
add rsp, 72
jmp .while_start_2
.while_end_2:
mov r9, qword [rbp - 40]
mov rax, 0
mov byte [r9], al
mov rdi, 1
call malloc
mov byte [rax + 0], 0
mov r15, rax
mov rax, r15
jmp .Lret_concat
.Lret_concat:
mov rsp, rbp
pop rbp
ret
extern printf, strlen, fopen, fclose, fread, fwrite, fseek, ftell, rewind
section .data
fmt_int: db "%d",0
fmt_char: db "%c",0
fmt_str: db "%s",0
fmt_float: db "%f",0
fmt_long: db "%ld",0
str_true: db "true",0
str_false: db "false",0
mode_write: db "w",0
mode_read: db "rb",0
section .text
global print_long
print_long:
    push rbp
    mov rbp, rsp
    sub rsp, 16         ;
    mov rsi, rdi
    mov rdi, fmt_long
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret
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
global print_str
print_str:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov rsi, rdi
    mov rdi, fmt_str            
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret
global print_fp
print_fp:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov rsi, rdi
    mov rdi, fmt_float
    xor rax, rax
    call printf
    add rsp, 16
    pop rbp
    ret
global write_file
write_file:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    push rbx
    push r12
    mov rbx, rdi
    mov r12, rsi
    mov rdi, rbx
    lea rsi, [rel mode_write]
    call fopen
    test rax, rax
    jz .error
    mov rbx, rax    
    mov rdi, r12
    call strlen
    mov rdi, r12    
    mov rsi, 1      
    mov rdx, rax   
    mov rcx, rbx    
    call fwrite
    mov rdi, rbx
    call fclose
    mov rax, 0
    jmp .cleanup
    .error:
    mov rax, -1
    .cleanup:
    pop r12
    pop rbx
    add rsp, 32
    pop rbp
    ret
global read_file
read_file:
    push rbp
    mov rbp, rsp
    sub rsp, 32

    add rsp, 32
    pop rbp
    ret


global file_size
file_size:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    push rbx
    
    lea rsi, [rel mode_read]    
    call fopen          
    test rax, rax
    jz .error
    mov rbx, rax
    
    mov rdi, rbx          
    xor rsi, rsi          
    mov rdx, 2            
    call fseek
    
    mov rdi, rbx
    call ftell
    mov rcx, rax         
    
    mov rdi, rbx  
    call fclose
    mov rax, rcx          
    jmp .done
    
.error:
    mov rax, -1
.done:
    pop rbx
    add rsp, 16
    pop rbp
    ret
extern malloc
extern free
extern exit
