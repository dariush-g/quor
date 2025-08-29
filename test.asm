global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rcx, 'e'
mov qword [rbp - 8], rcx
sub rsp, 8
mov edx, dword [rbp - 16]
mov rdi, rdx
call print_int
mov rbx, 0
xor rax, rax
mov rax, rbx
mov rdi, 10
call print_char
mov rdi, rbx
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
global char_at
char_at:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov dword [rbp - 12], esi
sub rsp, 8
mov r8, qword [rbp - 8]
mov r9, qword [rbp - 8]
mov r10d, dword [rbp - 12]
add r9, r10
mov qword [rbp - 24], r9
mov r11, qword [rbp - 24]
mov r12b, byte [r11]
xor rax, rax
mov rax, r12
jmp .Lret_char_at
.Lret_char_at:
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
mov r13, qword [rbp - 8]
mov rdi, r13
call strlen
mov qword [rbp - 24], rax
sub rsp, 8
mov r14, qword [rbp - 16]
mov rdi, r14
call strlen
mov qword [rbp - 32], rax
sub rsp, 8
mov r15d, dword [rbp - 24]
mov ecx, dword [rbp - 32]
add r15, rcx
mov rdx, 1
add r15, rdx
mov rdi, r15
call malloc
mov qword [rbp - 40], rax
mov rbx, qword [rbp - 40]
mov r10, ' '
mov byte [rbx], r10b
sub rsp, 8
mov r9, qword [rbp - 40]
mov qword [rbp - 48], r9
sub rsp, 8
mov r11, 0
mov qword [rbp - 56], r11
.while_start_0:
mov r12d, dword [rbp - 56]
mov r13d, dword [rbp - 24]
cmp r12, r13
setl al
movzx rax, al
cmp rax, 1
jne .while_end_0
mov r14, qword [rbp - 40]
mov rcx, qword [rbp - 8]
mov dl, byte [rcx]
mov byte [r14], dl
mov r15, qword [rbp - 40]
mov eax, 1
add r15, rax
mov qword [rbp - 40], r15
mov rbx, qword [rbp - 8]
mov r10d, 1
add rbx, r10
mov qword [rbp - 8], rbx
mov r9d, dword [rbp - 56]
mov r11, 1
add r9, r11
mov qword [rbp - 56], r9
add rsp, 56
jmp .while_start_0
.while_end_0:
mov dword [rbp - 56], 0
.while_start_1:
mov r12d, dword [rbp - 56]
mov r13d, dword [rbp - 32]
cmp r12, r13
setl al
movzx rax, al
cmp rax, 1
jne .while_end_1
mov rax, qword [rbp - 40]
mov rcx, qword [rbp - 16]
mov r14b, byte [rcx]
mov byte [rax], r14b
mov rdx, qword [rbp - 40]
mov eax, 1
add rdx, rax
mov qword [rbp - 40], rdx
mov r15, qword [rbp - 16]
mov r10d, 1
add r15, r10
mov qword [rbp - 16], r15
mov ebx, dword [rbp - 56]
mov r11, 1
add rbx, r11
mov qword [rbp - 56], rbx
add rsp, 56
jmp .while_start_1
.while_end_1:
mov r9, qword [rbp - 40]
mov r12, 0
mov byte [r9], r12b
mov r13, qword [rbp - 48]
xor rax, rax
mov rax, r13
jmp .Lret_concat
.Lret_concat:
mov rsp, rbp
pop rbp
ret
global contains_char
contains_char:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov byte [rbp - 9], sil
sub rsp, 8
mov rax, qword [rbp - 8]
mov rdi, rax
call strlen
mov qword [rbp - 24], rax
sub rsp, 8
mov rcx, 0
mov qword [rbp - 32], rcx
.while_start_2:
mov r14d, dword [rbp - 32]
mov eax, dword [rbp - 24]
cmp r14, rax
setl al
movzx rax, al
cmp rax, 1
jne .while_end_2
mov r10, qword [rbp - 8]
mov r15d, dword [rbp - 32]
mov rdi, r10
mov rsi, r15
call char_at
mov r11b, byte [rbp - 9]
cmp rax, r11
sete al
movzx rax, al
cmp rax, 0
je .else3
mov rbx, 1
.else3:
mov r9d, dword [rbp - 32]
mov r12, 1
add r9, r12
mov qword [rbp - 32], r9
add rsp, 32
jmp .while_start_2
.while_end_2:
mov r13, 0
xor rax, rax
mov rax, r13
jmp .Lret_contains_char
.Lret_contains_char:
mov rsp, rbp
pop rbp
ret
extern printf, strlen, fopen, fclose, fwrite, stat, rewind, fread, fseek, ftell, fflush
section .data
fmt_int: db "%d",0
fmt_char: db "%c",0
fmt_str: db "%s",0
fmt_float: db "%f",0
fmt_long: db "%ld",0
str_true: db "true",0
str_false: db "false",0
mode_write: db "w",0
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
global file_size
file_size:
    push rbp
    mov rbp, rsp
    sub rsp, 144             
    mov rsi, rsp             
    mov rdi, rdi             
    call stat               
    cmp eax, 0              
    jne .error
    mov rax, [rsp + 48]
    jmp .done
.error:
    mov rax, -1
.done:
    add rsp, 144
    pop rbp
    ret
global read_file
section .rodata
mode_rb: db "rb",0
section .text
read_file:
    push    r12
    push    rbp
    push    rbx
    lea     rsi, [rel mode_rb]
    call    fopen
    test    rax, rax
    je      .error
    mov     rbx, rax           
    mov     rdi, rbx
    mov     edx, 2           
    xor     esi, esi          
    call    fseek

    mov     rdi, rbx
    call    ftell
    mov     r12, rax           
    mov     rdi, rbx
    call    rewind

    lea     rdi, [r12+1]      
    call    malloc
    mov     rbp, rax         
    test    rax, rax
    je      .malloc_error

    mov     rdi, rbp          
    mov     rsi, 1            
    mov     rdx, r12          
    mov     rcx, rbx          
    call    fread

    mov     byte [rbp+r12], 0

    mov     rdi, rbx
    call    fclose

    mov     rax, rbp
    pop     rbx
    pop     rbp
    pop     r12
    ret

.malloc_error:
    mov     rdi, rbx
    call    fclose

.error:
    xor     eax, eax           
    pop     rbx
    pop     rbp
    pop     r12
    ret

extern malloc
extern free
extern exit
