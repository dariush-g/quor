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
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov rdi, rcx
call strlen
mov rdx, rax
mov qword [rbp - 16], rdx
mov rdi, 16
call malloc
mov byte [rax + 0], 'S'
mov byte [rax + 1], 't'
mov byte [rax + 2], 'r'
mov byte [rax + 3], 'i'
mov byte [rax + 4], 'n'
mov byte [rax + 5], 'g'
mov byte [rax + 6], ' '
mov byte [rax + 7], 'l'
mov byte [rax + 8], 'e'
mov byte [rax + 9], 'n'
mov byte [rax + 10], 'g'
mov byte [rax + 11], 't'
mov byte [rax + 12], 'h'
mov byte [rax + 13], ':'
mov byte [rax + 14], ' '
mov byte [rax + 15], 0
mov r12, rax
mov rdi, r12
call print_str
mov r8, rax
xor r9, r9
mov r9d, dword [rbp - 16]
mov rdi, r9
call print_int
mov r10, rax
mov r11, 10
mov rdi, r11
call print_char
mov r13, rax
sub rsp, 8
mov r14, 0
mov dword [rbp - 24], r14d
.while_start_0:
xor r15, r15
mov r15d, dword [rbp - 24]
xor rbx, rbx
mov ebx, dword [rbp - 16]
cmp r15, rbx
setl al
movzx rax, al
mov rcx, rax
cmp rcx, 1
jne .while_end_0
sub rsp, 8
xor rdx, rdx
mov rdx, qword [rbp - 8]
xor r12, r12
mov r12d, dword [rbp - 24]
mov rdi, rdx
mov rsi, r12
call char_at
mov r8, rax
mov qword [rbp - 32], r8
mov rdi, 7
call malloc
mov byte [rax + 0], 'I'
mov byte [rax + 1], 'n'
mov byte [rax + 2], 'd'
mov byte [rax + 3], 'e'
mov byte [rax + 4], 'x'
mov byte [rax + 5], ' '
mov byte [rax + 6], 0
mov r13, rax
mov rdi, r13
call print_str
mov r9, rax
xor r10, r10
mov r10d, dword [rbp - 24]
mov rdi, r10
call print_int
mov r11, rax
mov rdi, 3
call malloc
mov byte [rax + 0], ':'
mov byte [rax + 1], ' '
mov byte [rax + 2], 0
mov r14, rax
mov rdi, r14
call print_str
mov r15, rax
xor rbx, rbx
mov bl, byte [rbp - 32]
mov rdi, rbx
call print_char
mov rcx, rax
mov rdx, 10
mov rdi, rdx
call print_char
mov r12, rax
xor r8, r8
mov r8d, dword [rbp - 24]
mov r13, 1
add r8, r13
mov qword [rbp - 24], r8
jmp .while_start_0
.while_end_0:
mov rdi, 10
call print_char
mov rdi, rbx
mov r9, 0
xor rax, rax
mov rax, r9
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
xor r10, r10
mov r10, qword [rbp - 8]
xor r11, r11
mov r11, qword [rbp - 8]
xor r14, r14
mov r14d, dword [rbp - 12]
add r11, r14
mov qword [rbp - 24], r11
xor r15, r15
mov r15, qword [rbp - 24]
mov bl, byte [r15]
xor rax, rax
mov rax, rbx
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
mov rdx, 'a'
cmp rcx, rdx
setge al
movzx rax, al
mov r12, rax
xor r13, r13
mov r13b, byte [rbp - 1]
mov r8, 'z'
cmp r13, r8
setle al
movzx rax, al
mov r9, rax
cmp r12, 1
jne .and_end_1
cmp r9, 1
jne .and_end_1
mov rax, 1
jmp .and_done_2
.and_end_1:
mov rax, 0
.and_done_2:
mov r14, rax
cmp r14, 0
je .else2
mov r11, 1
xor rax, rax
mov rax, r11
jmp .Lret_is_alphabetic
.else2:
xor r15, r15
mov r15b, byte [rbp - 1]
mov rbx, 'A'
cmp r15, rbx
setge al
movzx rax, al
mov rcx, rax
xor rdx, rdx
mov dl, byte [rbp - 1]
mov r13, 'Z'
cmp rdx, r13
setle al
movzx rax, al
mov r8, rax
cmp rcx, 1
jne .and_end_3
cmp r8, 1
jne .and_end_3
mov rax, 1
jmp .and_done_4
.and_end_3:
mov rax, 0
.and_done_4:
mov r12, rax
cmp r12, 0
je .else4
mov r9, 1
xor rax, rax
mov rax, r9
jmp .Lret_is_alphabetic
.else4:
mov r14, 0
xor rax, rax
mov rax, r14
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
mov r15, rax
xor rbx, rbx
mov bl, byte [rbp - 1]
mov rdx, '0'
cmp rbx, rdx
setg al
movzx rax, al
mov r13, rax
xor rcx, rcx
mov cl, byte [rbp - 1]
mov r8, '9'
cmp rcx, r8
setl al
movzx rax, al
mov r12, rax
cmp r13, 1
jne .and_end_5
cmp r12, 1
jne .and_end_5
mov rax, 1
jmp .and_done_6
.and_end_5:
mov rax, 0
.and_done_6:
mov r9, rax
cmp r15, 1
je .or_end_6
cmp r9, 1
je .or_end_6
mov rax, 0
jmp .or_done_7
.or_end_6:
mov rax, 1
.or_done_7:
mov r14, rax
cmp r14, 0
je .else7
mov r11, 1
xor rax, rax
mov rax, r11
jmp .Lret_is_alphanumeric
.else7:
mov rbx, 0
xor rax, rax
mov rax, rbx
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
xor rdx, rdx
mov rdx, qword [rbp - 8]
mov rdi, rdx
call strlen
mov rcx, rax
mov qword [rbp - 24], rcx
sub rsp, 8
xor r8, r8
mov r8, qword [rbp - 16]
mov rdi, r8
call strlen
mov r13, rax
mov qword [rbp - 32], r13
xor r12, r12
mov r12d, dword [rbp - 24]
xor r15, r15
mov r15d, dword [rbp - 32]
add r12, r15
mov r9, 1
add r12, r9
mov rdi, r12
call malloc
mov r14, rax
sub rsp, 8
mov qword [rbp - 40], r14
sub rsp, 8
xor r11, r11
mov r11, qword [rbp - 40]
mov qword [rbp - 48], r11
sub rsp, 8
mov rbx, 0
mov dword [rbp - 56], ebx
.while_start_8:
xor rdx, rdx
mov edx, dword [rbp - 56]
xor rcx, rcx
mov ecx, dword [rbp - 24]
cmp rdx, rcx
setl al
movzx rax, al
mov r8, rax
cmp r8, 1
jne .while_end_8
xor r13, r13
mov r13, qword [rbp - 40]
xor r15, r15
mov r15, qword [rbp - 8]
mov r9b, byte [r15]
mov byte [r13], r9b
xor r12, r12
mov r12, qword [rbp - 40]
mov r14d, 1
add r12, r14
mov qword [rbp - 40], r12
xor r11, r11
mov r11, qword [rbp - 8]
mov ebx, 1
add r11, rbx
mov qword [rbp - 8], r11
xor rdx, rdx
mov edx, dword [rbp - 56]
mov rcx, 1
add rdx, rcx
mov qword [rbp - 56], rdx
jmp .while_start_8
.while_end_8:
mov dword [rbp - 56], 0
.while_start_9:
xor r8, r8
mov r8d, dword [rbp - 56]
xor r15, r15
mov r15d, dword [rbp - 32]
cmp r8, r15
setl al
movzx rax, al
mov r13, rax
cmp r13, 1
jne .while_end_9
xor r9, r9
mov r9, qword [rbp - 40]
xor r14, r14
mov r14, qword [rbp - 16]
mov r12b, byte [r14]
mov byte [r9], r12b
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov r11d, 1
add rbx, r11
mov qword [rbp - 40], rbx
xor rcx, rcx
mov rcx, qword [rbp - 16]
mov edx, 1
add rcx, rdx
mov qword [rbp - 16], rcx
xor r8, r8
mov r8d, dword [rbp - 56]
mov r15, 1
add r8, r15
mov qword [rbp - 56], r8
jmp .while_start_9
.while_end_9:
xor r13, r13
mov r13, qword [rbp - 40]
mov r14, 0
mov byte [r13], r14b
xor r9, r9
mov r9, qword [rbp - 48]
xor rax, rax
mov rax, r9
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
xor r12, r12
mov r12, qword [rbp - 8]
mov rdi, r12
call strlen
mov r11, rax
mov rbx, 1
add r11, rbx
mov rdi, r11
call malloc
mov rdx, rax
sub rsp, 8
mov qword [rbp - 24], rdx
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
xor r15, r15
mov r15, qword [rbp - 8]
mov rdi, r15
call strlen
mov r8, rax
mov qword [rbp - 24], r8
sub rsp, 8
mov r13, 0
mov dword [rbp - 32], r13d
.while_start_10:
xor r14, r14
mov r14d, dword [rbp - 32]
xor r9, r9
mov r9d, dword [rbp - 24]
cmp r14, r9
setl al
movzx rax, al
mov r12, rax
cmp r12, 1
jne .while_end_10
xor rbx, rbx
mov rbx, qword [rbp - 8]
xor r11, r11
mov r11d, dword [rbp - 32]
mov rdi, rbx
mov rsi, r11
call char_at
mov rdx, rax
xor rcx, rcx
mov cl, byte [rbp - 9]
cmp rdx, rcx
sete al
movzx rax, al
mov r15, rax
cmp r15, 0
je .else11
mov r8, 1
xor rax, rax
mov rax, r8
jmp .Lret_contains_char
.else11:
xor r13, r13
mov r13d, dword [rbp - 32]
mov r14, 1
add r13, r14
mov qword [rbp - 32], r13
jmp .while_start_10
.while_end_10:
mov r9, 0
xor rax, rax
mov rax, r9
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
