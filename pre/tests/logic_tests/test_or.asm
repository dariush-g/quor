global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rcx, 1
mov dword [rbp - 8], ecx
sub rsp, 8
mov rdx, 0
mov dword [rbp - 16], edx
sub rsp, 8
mov rbx, 1
mov dword [rbp - 24], ebx
xor r8, r8
mov r8d, dword [rbp - 8]
mov r9, 1
cmp r8, r9
sete al
movzx rax, al
mov r10, rax
xor r11, r11
mov r11d, dword [rbp - 16]
mov r12, 1
cmp r11, r12
sete al
movzx rax, al
mov r13, rax
cmp r10, 1
je .or_end_0
cmp r13, 1
je .or_end_0
mov rax, 0
jmp .or_done_1
.or_end_0:
mov rax, 1
.or_done_1:
mov r14, rax
xor r15, r15
mov r15d, dword [rbp - 24]
mov rcx, 1
cmp r15, rcx
sete al
movzx rax, al
mov rdx, rax
cmp r14, 1
je .or_end_1
cmp rdx, 1
je .or_end_1
mov rax, 0
jmp .or_done_2
.or_end_1:
mov rax, 1
.or_done_2:
mov rbx, rax
cmp rbx, 0
je .else2
mov rdi, 10
call malloc
mov byte [rax + 0], 'O'
mov byte [rax + 1], 'R'
mov byte [rax + 2], ' '
mov byte [rax + 3], 'w'
mov byte [rax + 4], 'o'
mov byte [rax + 5], 'r'
mov byte [rax + 6], 'k'
mov byte [rax + 7], 's'
mov byte [rax + 8], 10
mov byte [rax + 9], 0
mov r12, rax
mov rdi, r12
call print_str
mov r8, rax
jmp .endif2
.else2:
mov rdi, 11
call malloc
mov byte [rax + 0], 'O'
mov byte [rax + 1], 'R'
mov byte [rax + 2], ' '
mov byte [rax + 3], 'b'
mov byte [rax + 4], 'r'
mov byte [rax + 5], 'o'
mov byte [rax + 6], 'k'
mov byte [rax + 7], 'e'
mov byte [rax + 8], 'n'
mov byte [rax + 9], 10
mov byte [rax + 10], 0
mov r13, rax
mov rdi, r13
call print_str
mov r9, rax
.endif2:
xor r11, r11
mov r11d, dword [rbp - 8]
mov r10, 0
cmp r11, r10
sete al
movzx rax, al
mov r15, rax
xor rcx, rcx
mov ecx, dword [rbp - 16]
mov r14, 0
cmp rcx, r14
sete al
movzx rax, al
mov rdx, rax
cmp r15, 1
jne .and_end_3
cmp rdx, 1
jne .and_end_3
mov rax, 1
jmp .and_done_4
.and_end_3:
mov rax, 0
.and_done_4:
mov rbx, rax
xor r12, r12
mov r12d, dword [rbp - 24]
mov r8, 0
cmp r12, r8
sete al
movzx rax, al
mov r13, rax
cmp rbx, 1
jne .and_end_4
cmp r13, 1
jne .and_end_4
mov rax, 1
jmp .and_done_5
.and_end_4:
mov rax, 0
.and_done_5:
mov r9, rax
cmp r9, 0
je .else5
mov rdi, 12
call malloc
mov byte [rax + 0], 'A'
mov byte [rax + 1], 'N'
mov byte [rax + 2], 'D'
mov byte [rax + 3], ' '
mov byte [rax + 4], 'b'
mov byte [rax + 5], 'r'
mov byte [rax + 6], 'o'
mov byte [rax + 7], 'k'
mov byte [rax + 8], 'e'
mov byte [rax + 9], 'n'
mov byte [rax + 10], 10
mov byte [rax + 11], 0
mov r14, rax
mov rdi, r14
call print_str
mov r11, rax
jmp .endif5
.else5:
mov rdi, 11
call malloc
mov byte [rax + 0], 'A'
mov byte [rax + 1], 'N'
mov byte [rax + 2], 'D'
mov byte [rax + 3], ' '
mov byte [rax + 4], 'w'
mov byte [rax + 5], 'o'
mov byte [rax + 6], 'r'
mov byte [rax + 7], 'k'
mov byte [rax + 8], 's'
mov byte [rax + 9], 10
mov byte [rax + 10], 0
mov r15, rax
mov rdi, r15
call print_str
mov r10, rax
.endif5:
mov rdi, 10
call print_char
mov rdi, rbx
mov rcx, 0
xor rax, rax
mov rax, rcx
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
xor rdx, rdx
mov rdx, qword [rbp - 8]
xor r12, r12
mov r12, qword [rbp - 8]
xor r8, r8
mov r8d, dword [rbp - 12]
add r12, r8
mov qword [rbp - 24], r12
xor rbx, rbx
mov rbx, qword [rbp - 24]
mov r13b, byte [rbx]
xor rax, rax
mov rax, r13
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
mov r14, 'a'
cmp r9, r14
setge al
movzx rax, al
mov r11, rax
xor r15, r15
mov r15b, byte [rbp - 1]
mov r10, 'z'
cmp r15, r10
setle al
movzx rax, al
mov rcx, rax
cmp r11, 1
jne .and_end_6
cmp rcx, 1
jne .and_end_6
mov rax, 1
jmp .and_done_7
.and_end_6:
mov rax, 0
.and_done_7:
mov r8, rax
cmp r8, 0
je .else7
mov r12, 1
xor rax, rax
mov rax, r12
jmp .Lret_is_alphabetic
.else7:
xor rbx, rbx
mov bl, byte [rbp - 1]
mov r13, 'A'
cmp rbx, r13
setge al
movzx rax, al
mov r9, rax
xor r14, r14
mov r14b, byte [rbp - 1]
mov r15, 'Z'
cmp r14, r15
setle al
movzx rax, al
mov r10, rax
cmp r9, 1
jne .and_end_8
cmp r10, 1
jne .and_end_8
mov rax, 1
jmp .and_done_9
.and_end_8:
mov rax, 0
.and_done_9:
mov r11, rax
cmp r11, 0
je .else9
mov rcx, 1
xor rax, rax
mov rax, rcx
jmp .Lret_is_alphabetic
.else9:
mov r8, 0
xor rax, rax
mov rax, r8
jmp .Lret_is_alphabetic
.Lret_is_alphabetic:
mov rsp, rbp
pop rbp
ret
global is_alphanumeric
is_alphanumeric:
push rbp
mov rbp, rsp
sub rsp, 16
mov byte [rbp - 1], dil
xor r12, r12
mov r12b, byte [rbp - 1]
mov rdi, r12
call is_alphabetic
mov rbx, rax
xor r13, r13
mov r13b, byte [rbp - 1]
mov r14, '0'
cmp r13, r14
setg al
movzx rax, al
mov r15, rax
xor r9, r9
mov r9b, byte [rbp - 1]
mov r10, '9'
cmp r9, r10
setl al
movzx rax, al
mov r11, rax
cmp r15, 1
jne .and_end_10
cmp r11, 1
jne .and_end_10
mov rax, 1
jmp .and_done_11
.and_end_10:
mov rax, 0
.and_done_11:
mov rcx, rax
cmp rbx, 1
je .or_end_11
cmp rcx, 1
je .or_end_11
mov rax, 0
jmp .or_done_12
.or_end_11:
mov rax, 1
.or_done_12:
mov r8, rax
cmp r8, 0
je .else12
mov r12, 1
xor rax, rax
mov rax, r12
jmp .Lret_is_alphanumeric
.else12:
mov r13, 0
xor rax, rax
mov rax, r13
jmp .Lret_is_alphanumeric
.Lret_is_alphanumeric:
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
xor r14, r14
mov r14, qword [rbp - 8]
mov rdi, r14
call strlen
mov r9, rax
mov qword [rbp - 24], r9
sub rsp, 8
xor r10, r10
mov r10, qword [rbp - 16]
mov rdi, r10
call strlen
mov r15, rax
mov qword [rbp - 32], r15
xor r11, r11
mov r11d, dword [rbp - 24]
xor rbx, rbx
mov ebx, dword [rbp - 32]
add r11, rbx
mov rcx, 1
add r11, rcx
mov rdi, r11
call malloc
mov r8, rax
sub rsp, 8
mov qword [rbp - 40], r8
sub rsp, 8
xor r12, r12
mov r12, qword [rbp - 40]
mov qword [rbp - 48], r12
sub rsp, 8
mov r13, 0
mov dword [rbp - 56], r13d
.while_start_13:
xor r14, r14
mov r14d, dword [rbp - 56]
xor r9, r9
mov r9d, dword [rbp - 24]
cmp r14, r9
setl al
movzx rax, al
mov r10, rax
cmp r10, 1
jne .while_end_13
xor r15, r15
mov r15, qword [rbp - 40]
xor rbx, rbx
mov rbx, qword [rbp - 8]
mov cl, byte [rbx]
mov byte [r15], cl
xor r11, r11
mov r11, qword [rbp - 40]
mov r8d, 1
add r11, r8
mov qword [rbp - 40], r11
xor r12, r12
mov r12, qword [rbp - 8]
mov r13d, 1
add r12, r13
mov qword [rbp - 8], r12
xor r14, r14
mov r14d, dword [rbp - 56]
mov r9, 1
add r14, r9
mov qword [rbp - 56], r14
jmp .while_start_13
.while_end_13:
mov dword [rbp - 56], 0
.while_start_14:
xor r10, r10
mov r10d, dword [rbp - 56]
xor rbx, rbx
mov ebx, dword [rbp - 32]
cmp r10, rbx
setl al
movzx rax, al
mov r15, rax
cmp r15, 1
jne .while_end_14
xor rcx, rcx
mov rcx, qword [rbp - 40]
xor r8, r8
mov r8, qword [rbp - 16]
mov r11b, byte [r8]
mov byte [rcx], r11b
xor r13, r13
mov r13, qword [rbp - 40]
mov r12d, 1
add r13, r12
mov qword [rbp - 40], r13
xor r9, r9
mov r9, qword [rbp - 16]
mov r14d, 1
add r9, r14
mov qword [rbp - 16], r9
xor r10, r10
mov r10d, dword [rbp - 56]
mov rbx, 1
add r10, rbx
mov qword [rbp - 56], r10
jmp .while_start_14
.while_end_14:
xor r15, r15
mov r15, qword [rbp - 40]
mov r8, 0
mov byte [r15], r8b
xor rcx, rcx
mov rcx, qword [rbp - 48]
xor rax, rax
mov rax, rcx
jmp .Lret_concat
.Lret_concat:
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
xor r11, r11
mov r11, qword [rbp - 8]
mov rdi, r11
call strlen
mov r12, rax
mov r13, 1
add r12, r13
mov rdi, r12
call malloc
mov r14, rax
sub rsp, 8
mov qword [rbp - 24], r14
xor r9, r9
mov r9, qword [rbp - 24]
xor rax, rax
mov rax, r9
jmp .Lret_substring
.Lret_substring:
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
xor rbx, rbx
mov rbx, qword [rbp - 8]
mov rdi, rbx
call strlen
mov r10, rax
mov qword [rbp - 24], r10
sub rsp, 8
mov r15, 0
mov dword [rbp - 32], r15d
.while_start_15:
xor r8, r8
mov r8d, dword [rbp - 32]
xor rcx, rcx
mov ecx, dword [rbp - 24]
cmp r8, rcx
setl al
movzx rax, al
mov r11, rax
cmp r11, 1
jne .while_end_15
xor r13, r13
mov r13, qword [rbp - 8]
xor r12, r12
mov r12d, dword [rbp - 32]
mov rdi, r13
mov rsi, r12
call char_at
mov r14, rax
xor r9, r9
mov r9b, byte [rbp - 9]
cmp r14, r9
sete al
movzx rax, al
mov rbx, rax
cmp rbx, 0
je .else16
mov r10, 1
xor rax, rax
mov rax, r10
jmp .Lret_contains_char
.else16:
xor r15, r15
mov r15d, dword [rbp - 32]
mov r8, 1
add r15, r8
mov qword [rbp - 32], r15
jmp .while_start_15
.while_end_15:
mov rcx, 0
xor rax, rax
mov rax, rcx
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
