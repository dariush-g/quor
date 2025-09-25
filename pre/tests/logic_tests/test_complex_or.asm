global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rdi, 8
call malloc
mov byte [rax + 0], 't'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 's'
mov byte [rax + 3], 't'
mov byte [rax + 4], '.'
mov byte [rax + 5], 'q'
mov byte [rax + 6], 'u'
mov byte [rax + 7], 0
mov rbx, rax
mov qword [rbp - 8], rbx
sub rsp, 8
mov rcx, 6
mov dword [rbp - 16], ecx
sub rsp, 8
xor rdx, rdx
mov rdx, qword [rbp - 8]
xor r8, r8
mov r8d, dword [rbp - 16]
mov r9, 1
sub r8, r9
mov rdi, rdx
mov rsi, r8
call char_at
mov r10, rax
mov r11, 'u'
cmp r10, r11
setne al
movzx rax, al
mov r12, rax
mov qword [rbp - 24], r12
sub rsp, 8
xor r13, r13
mov r13, qword [rbp - 8]
xor r14, r14
mov r14d, dword [rbp - 16]
mov r15, 2
sub r14, r15
mov rdi, r13
mov rsi, r14
call char_at
mov rbx, rax
mov rcx, 'q'
cmp rbx, rcx
setne al
movzx rax, al
mov r9, rax
mov qword [rbp - 32], r9
sub rsp, 8
xor rdx, rdx
mov rdx, qword [rbp - 8]
xor r8, r8
mov r8d, dword [rbp - 16]
mov r10, 3
sub r8, r10
mov rdi, rdx
mov rsi, r8
call char_at
mov r11, rax
mov r12, '.'
cmp r11, r12
setne al
movzx rax, al
mov r15, rax
mov qword [rbp - 40], r15
sub rsp, 8
xor r13, r13
mov r13, qword [rbp - 8]
xor r14, r14
mov r14d, dword [rbp - 16]
mov rbx, 4
sub r14, rbx
mov rdi, r13
mov rsi, r14
call char_at
mov rcx, rax
mov rdi, rcx
call is_alphabetic
mov r9, rax
mov r10, 1
cmp r9, r10
setne al
movzx rax, al
mov rdx, rax
mov qword [rbp - 48], rdx
mov rdi, 8
call malloc
mov byte [rax + 0], 'c'
mov byte [rax + 1], 'o'
mov byte [rax + 2], 'n'
mov byte [rax + 3], 'd'
mov byte [rax + 4], '1'
mov byte [rax + 5], ':'
mov byte [rax + 6], ' '
mov byte [rax + 7], 0
mov r12, rax
mov rdi, r12
call print_str
mov r8, rax
xor r11, r11
mov r11b, byte [rbp - 24]
mov rdi, r11
call print_bool
mov r15, rax
mov rbx, 10
mov rdi, rbx
call print_char
mov r13, rax
mov rdi, 8
call malloc
mov byte [rax + 0], 'c'
mov byte [rax + 1], 'o'
mov byte [rax + 2], 'n'
mov byte [rax + 3], 'd'
mov byte [rax + 4], '2'
mov byte [rax + 5], ':'
mov byte [rax + 6], ' '
mov byte [rax + 7], 0
mov r14, rax
mov rdi, r14
call print_str
mov rcx, rax
xor r9, r9
mov r9b, byte [rbp - 32]
mov rdi, r9
call print_bool
mov r10, rax
mov rdx, 10
mov rdi, rdx
call print_char
mov r12, rax
mov rdi, 8
call malloc
mov byte [rax + 0], 'c'
mov byte [rax + 1], 'o'
mov byte [rax + 2], 'n'
mov byte [rax + 3], 'd'
mov byte [rax + 4], '3'
mov byte [rax + 5], ':'
mov byte [rax + 6], ' '
mov byte [rax + 7], 0
mov r15, rax
mov rdi, r15
call print_str
mov r8, rax
xor r11, r11
mov r11b, byte [rbp - 40]
mov rdi, r11
call print_bool
mov rbx, rax
mov r13, 10
mov rdi, r13
call print_char
mov r14, rax
mov rdi, 8
call malloc
mov byte [rax + 0], 'c'
mov byte [rax + 1], 'o'
mov byte [rax + 2], 'n'
mov byte [rax + 3], 'd'
mov byte [rax + 4], '4'
mov byte [rax + 5], ':'
mov byte [rax + 6], ' '
mov byte [rax + 7], 0
mov r12, rax
mov rdi, r12
call print_str
mov rcx, rax
xor r9, r9
mov r9b, byte [rbp - 48]
mov rdi, r9
call print_bool
mov r10, rax
mov rdx, 10
mov rdi, rdx
call print_char
mov r15, rax
xor r8, r8
mov r8b, byte [rbp - 24]
xor r11, r11
mov r11b, byte [rbp - 32]
cmp r8, 1
je .or_end_0
cmp r11, 1
je .or_end_0
mov rax, 0
jmp .or_done_1
.or_end_0:
mov rax, 1
.or_done_1:
mov rbx, rax
xor r13, r13
mov r13b, byte [rbp - 40]
cmp rbx, 1
je .or_end_1
cmp r13, 1
je .or_end_1
mov rax, 0
jmp .or_done_2
.or_end_1:
mov rax, 1
.or_done_2:
mov r14, rax
xor r12, r12
mov r12b, byte [rbp - 48]
cmp r14, 1
je .or_end_2
cmp r12, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov rcx, rax
cmp rcx, 0
je .else3
mov rdi, 22
call malloc
mov byte [rax + 0], 'O'
mov byte [rax + 1], 'R'
mov byte [rax + 2], ' '
mov byte [rax + 3], 'c'
mov byte [rax + 4], 'o'
mov byte [rax + 5], 'n'
mov byte [rax + 6], 'd'
mov byte [rax + 7], 'i'
mov byte [rax + 8], 't'
mov byte [rax + 9], 'i'
mov byte [rax + 10], 'o'
mov byte [rax + 11], 'n'
mov byte [rax + 12], ' '
mov byte [rax + 13], 'i'
mov byte [rax + 14], 's'
mov byte [rax + 15], ' '
mov byte [rax + 16], 'T'
mov byte [rax + 17], 'R'
mov byte [rax + 18], 'U'
mov byte [rax + 19], 'E'
mov byte [rax + 20], 10
mov byte [rax + 21], 0
mov r15, rax
mov rdi, r15
call print_str
mov r9, rax
jmp .endif3
.else3:
mov rdi, 23
call malloc
mov byte [rax + 0], 'O'
mov byte [rax + 1], 'R'
mov byte [rax + 2], ' '
mov byte [rax + 3], 'c'
mov byte [rax + 4], 'o'
mov byte [rax + 5], 'n'
mov byte [rax + 6], 'd'
mov byte [rax + 7], 'i'
mov byte [rax + 8], 't'
mov byte [rax + 9], 'i'
mov byte [rax + 10], 'o'
mov byte [rax + 11], 'n'
mov byte [rax + 12], ' '
mov byte [rax + 13], 'i'
mov byte [rax + 14], 's'
mov byte [rax + 15], ' '
mov byte [rax + 16], 'F'
mov byte [rax + 17], 'A'
mov byte [rax + 18], 'L'
mov byte [rax + 19], 'S'
mov byte [rax + 20], 'E'
mov byte [rax + 21], 10
mov byte [rax + 22], 0
mov rbx, rax
mov rdi, rbx
call print_str
mov r10, rax
.endif3:
mov rdi, 10
call print_char
mov rdi, rbx
mov rdx, 0
xor rax, rax
mov rax, rdx
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
xor r8, r8
mov r8, qword [rbp - 8]
xor r11, r11
mov r11, qword [rbp - 8]
xor r13, r13
mov r13d, dword [rbp - 12]
add r11, r13
mov qword [rbp - 24], r11
xor r14, r14
mov r14, qword [rbp - 24]
mov r12b, byte [r14]
xor rax, rax
mov rax, r12
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
xor rcx, rcx
mov cl, byte [rbp - 1]
mov r15, 'a'
cmp rcx, r15
setge al
movzx rax, al
mov r9, rax
xor rbx, rbx
mov bl, byte [rbp - 1]
mov r10, 'z'
cmp rbx, r10
setle al
movzx rax, al
mov rdx, rax
cmp r9, 1
jne .and_end_4
cmp rdx, 1
jne .and_end_4
mov rax, 1
jmp .and_done_5
.and_end_4:
mov rax, 0
.and_done_5:
mov r13, rax
cmp r13, 0
je .else5
mov r11, 1
xor rax, rax
mov rax, r11
jmp .Lret_is_alphabetic
.else5:
xor r14, r14
mov r14b, byte [rbp - 1]
mov r12, 'A'
cmp r14, r12
setge al
movzx rax, al
mov rcx, rax
xor r15, r15
mov r15b, byte [rbp - 1]
mov rbx, 'Z'
cmp r15, rbx
setle al
movzx rax, al
mov r10, rax
cmp rcx, 1
jne .and_end_6
cmp r10, 1
jne .and_end_6
mov rax, 1
jmp .and_done_7
.and_end_6:
mov rax, 0
.and_done_7:
mov r9, rax
cmp r9, 0
je .else7
mov rdx, 1
xor rax, rax
mov rax, rdx
jmp .Lret_is_alphabetic
.else7:
mov r13, 0
xor rax, rax
mov rax, r13
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
xor r11, r11
mov r11b, byte [rbp - 1]
mov rdi, r11
call is_alphabetic
mov r14, rax
xor r12, r12
mov r12b, byte [rbp - 1]
mov r15, '0'
cmp r12, r15
setg al
movzx rax, al
mov rbx, rax
xor rcx, rcx
mov cl, byte [rbp - 1]
mov r10, '9'
cmp rcx, r10
setl al
movzx rax, al
mov r9, rax
cmp rbx, 1
jne .and_end_8
cmp r9, 1
jne .and_end_8
mov rax, 1
jmp .and_done_9
.and_end_8:
mov rax, 0
.and_done_9:
mov rdx, rax
cmp r14, 1
je .or_end_9
cmp rdx, 1
je .or_end_9
mov rax, 0
jmp .or_done_10
.or_end_9:
mov rax, 1
.or_done_10:
mov r13, rax
cmp r13, 0
je .else10
mov r11, 1
xor rax, rax
mov rax, r11
jmp .Lret_is_alphanumeric
.else10:
mov r12, 0
xor rax, rax
mov rax, r12
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
xor r15, r15
mov r15, qword [rbp - 8]
mov rdi, r15
call strlen
mov rcx, rax
mov qword [rbp - 24], rcx
sub rsp, 8
xor r10, r10
mov r10, qword [rbp - 16]
mov rdi, r10
call strlen
mov rbx, rax
mov qword [rbp - 32], rbx
xor r9, r9
mov r9d, dword [rbp - 24]
xor r14, r14
mov r14d, dword [rbp - 32]
add r9, r14
mov rdx, 1
add r9, rdx
mov rdi, r9
call malloc
mov r13, rax
sub rsp, 8
mov qword [rbp - 40], r13
sub rsp, 8
xor r11, r11
mov r11, qword [rbp - 40]
mov qword [rbp - 48], r11
sub rsp, 8
mov r12, 0
mov dword [rbp - 56], r12d
.while_start_11:
xor r15, r15
mov r15d, dword [rbp - 56]
xor rcx, rcx
mov ecx, dword [rbp - 24]
cmp r15, rcx
setl al
movzx rax, al
mov r10, rax
cmp r10, 1
jne .while_end_11
xor rbx, rbx
mov rbx, qword [rbp - 40]
xor r14, r14
mov r14, qword [rbp - 8]
mov dl, byte [r14]
mov byte [rbx], dl
xor r9, r9
mov r9, qword [rbp - 40]
mov r13d, 1
add r9, r13
mov qword [rbp - 40], r9
xor r11, r11
mov r11, qword [rbp - 8]
mov r12d, 1
add r11, r12
mov qword [rbp - 8], r11
xor r15, r15
mov r15d, dword [rbp - 56]
mov rcx, 1
add r15, rcx
mov qword [rbp - 56], r15
jmp .while_start_11
.while_end_11:
mov dword [rbp - 56], 0
.while_start_12:
xor r10, r10
mov r10d, dword [rbp - 56]
xor r14, r14
mov r14d, dword [rbp - 32]
cmp r10, r14
setl al
movzx rax, al
mov rbx, rax
cmp rbx, 1
jne .while_end_12
xor rdx, rdx
mov rdx, qword [rbp - 40]
xor r13, r13
mov r13, qword [rbp - 16]
mov r9b, byte [r13]
mov byte [rdx], r9b
xor r12, r12
mov r12, qword [rbp - 40]
mov r11d, 1
add r12, r11
mov qword [rbp - 40], r12
xor rcx, rcx
mov rcx, qword [rbp - 16]
mov r15d, 1
add rcx, r15
mov qword [rbp - 16], rcx
xor r10, r10
mov r10d, dword [rbp - 56]
mov r14, 1
add r10, r14
mov qword [rbp - 56], r10
jmp .while_start_12
.while_end_12:
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov r13, 0
mov byte [rbx], r13b
xor rdx, rdx
mov rdx, qword [rbp - 48]
xor rax, rax
mov rax, rdx
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
xor r9, r9
mov r9, qword [rbp - 8]
mov rdi, r9
call strlen
mov r11, rax
mov r12, 1
add r11, r12
mov rdi, r11
call malloc
mov r15, rax
sub rsp, 8
mov qword [rbp - 24], r15
xor rcx, rcx
mov rcx, qword [rbp - 24]
xor rax, rax
mov rax, rcx
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
xor r14, r14
mov r14, qword [rbp - 8]
mov rdi, r14
call strlen
mov r10, rax
mov qword [rbp - 24], r10
sub rsp, 8
mov rbx, 0
mov dword [rbp - 32], ebx
.while_start_13:
xor r13, r13
mov r13d, dword [rbp - 32]
xor rdx, rdx
mov edx, dword [rbp - 24]
cmp r13, rdx
setl al
movzx rax, al
mov r9, rax
cmp r9, 1
jne .while_end_13
xor r12, r12
mov r12, qword [rbp - 8]
xor r11, r11
mov r11d, dword [rbp - 32]
mov rdi, r12
mov rsi, r11
call char_at
mov r15, rax
xor rcx, rcx
mov cl, byte [rbp - 9]
cmp r15, rcx
sete al
movzx rax, al
mov r14, rax
cmp r14, 0
je .else14
mov r10, 1
xor rax, rax
mov rax, r10
jmp .Lret_contains_char
.else14:
xor rbx, rbx
mov ebx, dword [rbp - 32]
mov r13, 1
add rbx, r13
mov qword [rbp - 32], rbx
jmp .while_start_13
.while_end_13:
mov rdx, 0
xor rax, rax
mov rax, rdx
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
