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
cmp r12, 1
je .or_end_0
cmp r9, 1
je .or_end_0
mov rax, 0
jmp .or_done_1
.or_end_0:
mov rax, 1
.or_done_1:
mov rdx, rax
xor r8, r8
mov r8, qword [rbp - 8]
xor r10, r10
mov r10d, dword [rbp - 16]
mov r11, 3
sub r10, r11
mov rdi, r8
mov rsi, r10
call char_at
mov r15, rax
mov r13, '.'
cmp r15, r13
setne al
movzx rax, al
mov r14, rax
cmp rdx, 1
je .or_end_1
cmp r14, 1
je .or_end_1
mov rax, 0
jmp .or_done_2
.or_end_1:
mov rax, 1
.or_done_2:
mov rbx, rax
xor rcx, rcx
mov rcx, qword [rbp - 8]
xor r12, r12
mov r12d, dword [rbp - 16]
mov r9, 4
sub r12, r9
mov rdi, rcx
mov rsi, r12
call char_at
mov r11, rax
mov rdi, r11
call is_alphabetic
mov r8, rax
mov r10, 1
cmp r8, r10
setne al
movzx rax, al
mov r15, rax
cmp rbx, 1
je .or_end_2
cmp r15, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov r13, rax
cmp r13, 0
je .else3
mov rdi, 31
call malloc
mov byte [rax + 0], 'E'
mov byte [rax + 1], 'x'
mov byte [rax + 2], 'p'
mov byte [rax + 3], 'e'
mov byte [rax + 4], 'c'
mov byte [rax + 5], 't'
mov byte [rax + 6], 'e'
mov byte [rax + 7], 'd'
mov byte [rax + 8], ' '
mov byte [rax + 9], '<'
mov byte [rax + 10], 'f'
mov byte [rax + 11], 'i'
mov byte [rax + 12], 'l'
mov byte [rax + 13], 'e'
mov byte [rax + 14], 'n'
mov byte [rax + 15], 'a'
mov byte [rax + 16], 'm'
mov byte [rax + 17], 'e'
mov byte [rax + 18], '>'
mov byte [rax + 19], '.'
mov byte [rax + 20], 'q'
mov byte [rax + 21], 'u'
mov byte [rax + 22], ' '
mov byte [rax + 23], '|'
mov byte [rax + 24], ' '
mov byte [rax + 25], 'G'
mov byte [rax + 26], 'o'
mov byte [rax + 27], 't'
mov byte [rax + 28], ':'
mov byte [rax + 29], ' '
mov byte [rax + 30], 0
mov r14, rax
mov rdi, r14
call print_str
mov rdx, rax
xor r9, r9
mov r9, qword [rbp - 8]
mov rdi, r9
call print_str
mov rcx, rax
mov r12, 10
mov rdi, r12
call print_char
mov r11, rax
mov r8, 1
mov rdi, r8
call exit
mov r10, rax
jmp .endif3
.else3:
mov rdi, 25
call malloc
mov byte [rax + 0], 'F'
mov byte [rax + 1], 'i'
mov byte [rax + 2], 'l'
mov byte [rax + 3], 'e'
mov byte [rax + 4], ' '
mov byte [rax + 5], 'e'
mov byte [rax + 6], 'x'
mov byte [rax + 7], 't'
mov byte [rax + 8], 'e'
mov byte [rax + 9], 'n'
mov byte [rax + 10], 's'
mov byte [rax + 11], 'i'
mov byte [rax + 12], 'o'
mov byte [rax + 13], 'n'
mov byte [rax + 14], ' '
mov byte [rax + 15], 'i'
mov byte [rax + 16], 's'
mov byte [rax + 17], ' '
mov byte [rax + 18], 'v'
mov byte [rax + 19], 'a'
mov byte [rax + 20], 'l'
mov byte [rax + 21], 'i'
mov byte [rax + 22], 'd'
mov byte [rax + 23], 10
mov byte [rax + 24], 0
mov rbx, rax
mov rdi, rbx
call print_str
mov r15, rax
.endif3:
mov rdi, 10
call print_char
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
xor rdx, rdx
mov rdx, qword [rbp - 8]
xor r9, r9
mov r9d, dword [rbp - 12]
add rdx, r9
mov qword [rbp - 24], rdx
xor rcx, rcx
mov rcx, qword [rbp - 24]
mov r12b, byte [rcx]
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
xor r11, r11
mov r11b, byte [rbp - 1]
mov r8, 'a'
cmp r11, r8
setge al
movzx rax, al
mov r10, rax
xor rbx, rbx
mov bl, byte [rbp - 1]
mov r15, 'z'
cmp rbx, r15
setle al
movzx rax, al
mov r13, rax
cmp r10, 1
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
mov rdx, 1
xor rax, rax
mov rax, rdx
jmp .Lret_is_alphabetic
.else5:
xor rcx, rcx
mov cl, byte [rbp - 1]
mov r12, 'A'
cmp rcx, r12
setge al
movzx rax, al
mov r11, rax
xor r8, r8
mov r8b, byte [rbp - 1]
mov rbx, 'Z'
cmp r8, rbx
setle al
movzx rax, al
mov r15, rax
cmp r11, 1
jne .and_end_6
cmp r15, 1
jne .and_end_6
mov rax, 1
jmp .and_done_7
.and_end_6:
mov rax, 0
.and_done_7:
mov r10, rax
cmp r10, 0
je .else7
mov r13, 1
xor rax, rax
mov rax, r13
jmp .Lret_is_alphabetic
.else7:
mov r9, 0
xor rax, rax
mov rax, r9
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
xor rdx, rdx
mov dl, byte [rbp - 1]
mov rdi, rdx
call is_alphabetic
mov rcx, rax
xor r12, r12
mov r12b, byte [rbp - 1]
mov r8, '0'
cmp r12, r8
setg al
movzx rax, al
mov rbx, rax
xor r11, r11
mov r11b, byte [rbp - 1]
mov r15, '9'
cmp r11, r15
setl al
movzx rax, al
mov r10, rax
cmp rbx, 1
jne .and_end_8
cmp r10, 1
jne .and_end_8
mov rax, 1
jmp .and_done_9
.and_end_8:
mov rax, 0
.and_done_9:
mov r13, rax
cmp rcx, 1
je .or_end_9
cmp r13, 1
je .or_end_9
mov rax, 0
jmp .or_done_10
.or_end_9:
mov rax, 1
.or_done_10:
mov r9, rax
cmp r9, 0
je .else10
mov rdx, 1
xor rax, rax
mov rax, rdx
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
xor r8, r8
mov r8, qword [rbp - 8]
mov rdi, r8
call strlen
mov r11, rax
mov qword [rbp - 24], r11
sub rsp, 8
xor r15, r15
mov r15, qword [rbp - 16]
mov rdi, r15
call strlen
mov rbx, rax
mov qword [rbp - 32], rbx
xor r10, r10
mov r10d, dword [rbp - 24]
xor rcx, rcx
mov ecx, dword [rbp - 32]
add r10, rcx
mov r13, 1
add r10, r13
mov rdi, r10
call malloc
mov r9, rax
sub rsp, 8
mov qword [rbp - 40], r9
sub rsp, 8
xor rdx, rdx
mov rdx, qword [rbp - 40]
mov qword [rbp - 48], rdx
sub rsp, 8
mov r12, 0
mov dword [rbp - 56], r12d
.while_start_11:
xor r8, r8
mov r8d, dword [rbp - 56]
xor r11, r11
mov r11d, dword [rbp - 24]
cmp r8, r11
setl al
movzx rax, al
mov r15, rax
cmp r15, 1
jne .while_end_11
xor rbx, rbx
mov rbx, qword [rbp - 40]
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov r13b, byte [rcx]
mov byte [rbx], r13b
xor r10, r10
mov r10, qword [rbp - 40]
mov r9d, 1
add r10, r9
mov qword [rbp - 40], r10
xor rdx, rdx
mov rdx, qword [rbp - 8]
mov r12d, 1
add rdx, r12
mov qword [rbp - 8], rdx
xor r8, r8
mov r8d, dword [rbp - 56]
mov r11, 1
add r8, r11
mov qword [rbp - 56], r8
jmp .while_start_11
.while_end_11:
mov dword [rbp - 56], 0
.while_start_12:
xor r15, r15
mov r15d, dword [rbp - 56]
xor rcx, rcx
mov ecx, dword [rbp - 32]
cmp r15, rcx
setl al
movzx rax, al
mov rbx, rax
cmp rbx, 1
jne .while_end_12
xor r13, r13
mov r13, qword [rbp - 40]
xor r9, r9
mov r9, qword [rbp - 16]
mov r10b, byte [r9]
mov byte [r13], r10b
xor r12, r12
mov r12, qword [rbp - 40]
mov edx, 1
add r12, rdx
mov qword [rbp - 40], r12
xor r11, r11
mov r11, qword [rbp - 16]
mov r8d, 1
add r11, r8
mov qword [rbp - 16], r11
xor r15, r15
mov r15d, dword [rbp - 56]
mov rcx, 1
add r15, rcx
mov qword [rbp - 56], r15
jmp .while_start_12
.while_end_12:
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov r9, 0
mov byte [rbx], r9b
xor r13, r13
mov r13, qword [rbp - 48]
xor rax, rax
mov rax, r13
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
xor r10, r10
mov r10, qword [rbp - 8]
mov rdi, r10
call strlen
mov rdx, rax
mov r12, 1
add rdx, r12
mov rdi, rdx
call malloc
mov r8, rax
sub rsp, 8
mov qword [rbp - 24], r8
xor r11, r11
mov r11, qword [rbp - 24]
xor rax, rax
mov rax, r11
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
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov rdi, rcx
call strlen
mov r15, rax
mov qword [rbp - 24], r15
sub rsp, 8
mov rbx, 0
mov dword [rbp - 32], ebx
.while_start_13:
xor r9, r9
mov r9d, dword [rbp - 32]
xor r13, r13
mov r13d, dword [rbp - 24]
cmp r9, r13
setl al
movzx rax, al
mov r10, rax
cmp r10, 1
jne .while_end_13
xor r12, r12
mov r12, qword [rbp - 8]
xor rdx, rdx
mov edx, dword [rbp - 32]
mov rdi, r12
mov rsi, rdx
call char_at
mov r8, rax
xor r11, r11
mov r11b, byte [rbp - 9]
cmp r8, r11
sete al
movzx rax, al
mov rcx, rax
cmp rcx, 0
je .else14
mov r15, 1
xor rax, rax
mov rax, r15
jmp .Lret_contains_char
.else14:
xor rbx, rbx
mov ebx, dword [rbp - 32]
mov r9, 1
add rbx, r9
mov qword [rbp - 32], rbx
jmp .while_start_13
.while_end_13:
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
