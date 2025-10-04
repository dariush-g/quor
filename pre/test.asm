section .bss

section .rodata

section .text
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rdi, 6
sub rsp, 8
call malloc
add rsp, 8
mov byte [rax + 0], 'h'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 'l'
mov byte [rax + 3], 'l'
mov byte [rax + 4], 'o'
mov byte [rax + 5], 0
mov rbx, rax
mov qword [rbp - 8], rbx
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov rdx, 1
mov r8, 3
mov rdi, rcx
mov rsi, rdx
mov rdx, r8
sub rsp, 8
call substring
add rsp, 8
mov r9, rax
mov rdi, r9
sub rsp, 8
call print_str
add rsp, 8
mov r10, rax
xor r11, r11
mov r11, qword [rbp - 8]
mov rdi, r11
sub rsp, 8
call print_str
add rsp, 8
mov r12, rax
mov rdi, 10
sub rsp, 8
call print_char
add rsp, 8
mov rdi, rbx
mov r13, 0
xor rax, rax
mov rax, r13
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
xor r14, r14
mov r14, qword [rbp - 8]
xor r15, r15
mov r15, qword [rbp - 8]
sub rsp, 8
mov qword [rsp], r15
xor rbx, rbx
mov ebx, dword [rbp - 12]
mov rcx, qword [rsp]
add rsp, 8
add ecx, ebx
mov qword [rbp - 24], rcx
xor rdx, rdx
mov rdx, qword [rbp - 24]
mov r8b, byte [rdx]
xor rax, rax
mov rax, r8
jmp .Lret_char_at
.Lret_char_at:
mov rsp, rbp
pop rbp
ret
global is_alphabetic
is_alphabetic:
push rbp
mov rbp, rsp
sub rsp, 16
mov byte [rbp - 1], dil
xor r9, r9
mov r9b, byte [rbp - 1]
sub rsp, 8
mov qword [rsp], r9
mov r10, 'a'
mov r11, qword [rsp]
add rsp, 8
cmp r11, r10
setge al
movzx rax, al
mov r12, rax
sub rsp, 8
mov qword [rsp], r12
xor r13, r13
mov r13b, byte [rbp - 1]
sub rsp, 8
mov qword [rsp], r13
mov r15, 'z'
mov rbx, qword [rsp]
add rsp, 8
cmp rbx, r15
setle al
movzx rax, al
mov rcx, rax
mov rdx, qword [rsp]
add rsp, 8
cmp rdx, 1
jne .and_end_0
cmp rcx, 1
jne .and_end_0
mov rax, 1
jmp .and_done_1
.and_end_0:
mov rax, 0
.and_done_1:
mov r8, rax
cmp r8, 0
je .else1
mov r9, 1
xor rax, rax
mov rax, r9
jmp .Lret_is_alphabetic
.else1:
xor r11, r11
mov r11b, byte [rbp - 1]
sub rsp, 8
mov qword [rsp], r11
mov r10, 'A'
mov r12, qword [rsp]
add rsp, 8
cmp r12, r10
setge al
movzx rax, al
mov r13, rax
sub rsp, 8
mov qword [rsp], r13
xor rbx, rbx
mov bl, byte [rbp - 1]
sub rsp, 8
mov qword [rsp], rbx
mov r15, 'Z'
mov rdx, qword [rsp]
add rsp, 8
cmp rdx, r15
setle al
movzx rax, al
mov rcx, rax
mov r8, qword [rsp]
add rsp, 8
cmp r8, 1
jne .and_end_2
cmp rcx, 1
jne .and_end_2
mov rax, 1
jmp .and_done_3
.and_end_2:
mov rax, 0
.and_done_3:
mov r9, rax
cmp r9, 0
je .else3
mov r11, 1
xor rax, rax
mov rax, r11
jmp .Lret_is_alphabetic
.else3:
mov r12, 0
xor rax, rax
mov rax, r12
jmp .Lret_is_alphabetic
.Lret_is_alphabetic:
mov rsp, rbp
pop rbp
ret
global substring
substring:
push rbp
mov rbp, rsp
sub rsp, 16
mov qword [rbp - 8], rdi
mov dword [rbp - 12], esi
mov dword [rbp - 16], edx
xor r10, r10
mov r10d, dword [rbp - 16]
sub rsp, 8
mov qword [rsp], r10
xor r13, r13
mov r13d, dword [rbp - 12]
mov rbx, qword [rsp]
add rsp, 8
sub ebx, r13d
sub rsp, 8
mov qword [rsp], rbx
mov rdx, 1
mov r15, qword [rsp]
add rsp, 8
add r15d, edx
mov rdi, r15
call malloc
mov r8, rax
sub rsp, 8
mov qword [rbp - 24], r8
sub rsp, 8
xor rcx, rcx
mov rcx, qword [rbp - 24]
mov qword [rbp - 32], rcx
xor r9, r9
mov r9, qword [rbp - 8]
xor r11, r11
mov r11, qword [rbp - 8]
sub rsp, 8
mov qword [rsp], r11
xor r12, r12
mov r12d, dword [rbp - 12]
mov r10, qword [rsp]
add rsp, 8
add r10d, r12d
mov qword [rbp - 8], r10
sub rsp, 8
mov r13, 0
mov dword [rbp - 40], r13d
.while_start_4:
xor rbx, rbx
mov ebx, dword [rbp - 40]
sub rsp, 8
mov qword [rsp], rbx
xor rdx, rdx
mov edx, dword [rbp - 16]
sub rsp, 8
mov qword [rsp], rdx
xor r15, r15
mov r15d, dword [rbp - 12]
mov r8, qword [rsp]
add rsp, 8
sub r8d, r15d
mov rcx, qword [rsp]
add rsp, 8
cmp rcx, r8
setl al
movzx rax, al
mov r11, rax
cmp r11, 1
jne .while_end_4
xor r12, r12
mov r12, qword [rbp - 24]
xor r10, r10
mov r10, qword [rbp - 8]
mov r13b, byte [r10]
mov byte [r12], r13b
xor rbx, rbx
mov rbx, qword [rbp - 24]
mov edx, 1
add rbx, rdx
mov qword [rbp - 24], rbx
xor r15, r15
mov r15, qword [rbp - 8]
mov ecx, 1
add r15, rcx
mov qword [rbp - 8], r15
mov r8d, dword [rbp - 40]
inc dword [rbp - 40]
jmp .while_start_4
.while_end_4:
xor r11, r11
mov r11, qword [rbp - 24]
mov r10, 0
mov byte [r11], r10b
xor r12, r12
mov r12, qword [rbp - 32]
xor rax, rax
mov rax, r12
jmp .Lret_substring
.Lret_substring:
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
