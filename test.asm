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
mov r8, qword [rbp - 24]
mov rdi, r8
call print_str
mov r9, 0
mov rax, r9
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
global strlen
strlen:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
sub rsp, 8
mov r10, 0
mov qword [rbp - 24], r10
.while_start_0:
mov r11, qword [rbp - 8]
mov r11, qword [r11]
mov r13, 0
cmp r11, r13
setne al
movzx rax, al
cmp rax, 1
jne .while_end_0
mov r14d, dword [rbp - 24]
mov r15, 1
add r14, r15
mov qword [rbp - 24], r14
mov rbx, qword [rbp - 8]
mov r12d, 1
add rbx, r12
mov qword [rbp - 8], rbx
add rsp, 24
jmp .while_start_0
.while_end_0:
mov rcx, qword [rbp - 8]
mov rdx, qword [rbp - 8]
mov r8d, dword [rbp - 24]
sub rdx, r8
mov qword [rbp - 8], rdx
mov eax, dword [rbp - 24]
jmp .Lret_strlen
.Lret_strlen:
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
mov r9, qword [rbp - 8]
mov rdi, r9
call strlen
mov qword [rbp - 24], rax
sub rsp, 8
mov r10, qword [rbp - 16]
mov rdi, r10
call strlen
mov qword [rbp - 32], rax
sub rsp, 8
mov r11d, dword [rbp - 24]
mov r13d, dword [rbp - 32]
add r11, r13
mov r15, 1
add r11, r15
mov rdi, r11
call malloc
mov qword [rbp - 40], rax
mov r14, qword [rbp - 40]
mov r12, ' '
mov byte [r14], r12b
sub rsp, 8
mov rbx, qword [rbp - 40]
mov qword [rbp - 48], rbx
sub rsp, 8
mov r8d, dword [rbp - 24]
mov qword [rbp - 56], r8
.while_start_1:
mov edx, dword [rbp - 56]
mov r9, 0
cmp rdx, r9
setg al
movzx rax, al
cmp rax, 1
jne .while_end_1
mov r10, qword [rbp - 40]
mov r13, qword [rbp - 8]
mov r13, qword [r13]
mov qword [r10], r13
mov r15, qword [rbp - 40]
mov r11d, 1
add r15, r11
mov qword [rbp - 40], r15
mov rax, qword [rbp - 8]
mov r14d, 1
add rax, r14
mov qword [rbp - 8], rax
add rsp, 56
jmp .while_start_1
.while_end_1:
mov r12, qword [rbp - 8]
mov rbx, qword [rbp - 8]
mov r8d, dword [rbp - 56]
sub rbx, r8
mov qword [rbp - 8], rbx
sub rsp, 8
mov edx, dword [rbp - 32]
mov qword [rbp - 64], rdx
.while_start_2:
mov r9d, dword [rbp - 64]
mov rax, 0
cmp r9, rax
setg al
movzx rax, al
cmp rax, 1
jne .while_end_2
mov r10, qword [rbp - 40]
mov r13, qword [rbp - 16]
mov r13, qword [r13]
mov qword [r10], r13
mov r11, qword [rbp - 40]
mov r15d, 1
add r11, r15
mov qword [rbp - 40], r11
mov r14, qword [rbp - 16]
mov eax, 1
add r14, rax
mov qword [rbp - 16], r14
add rsp, 64
jmp .while_start_2
.while_end_2:
mov r8, qword [rbp - 16]
mov rbx, qword [rbp - 16]
mov edx, dword [rbp - 56]
sub rbx, rdx
mov qword [rbp - 16], rbx
mov r9, qword [rbp - 40]
mov eax, 1
add r9, rax
mov qword [rbp - 40], r9
mov rax, qword [rbp - 40]
mov r10, 0
mov byte [rax], r10b
mov r13, qword [rbp - 48]
mov rax, r13
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
