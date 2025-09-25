section .data
TOKEN_TYPE_INT_LIT: dd 0
section .text
section .data
TOKEN_TYPE_CHAR_LIT: dd 1
section .text
section .data
TOKEN_TYPE_BOOL_LIT: dd 2
section .text
section .data
TOKEN_TYPE_FLOAT_LIT: dd 3
section .text
section .data
TOKEN_TYPE_STRING_LIT: dd 4
section .text
section .data
TOKEN_TYPE_LONG_LIT: dd 5
section .text
section .data
TOKEN_TYPE_DOUBLE_LIT: dd 6
section .text
section .data
TOKEN_TYPE_STRUCT: dd 7
section .text
section .data
TOKEN_TYPE_LET: dd 8
section .text
section .data
TOKEN_TYPE_DEF: dd 9
section .text
section .data
TOKEN_TYPE_WHILE: dd 10
section .text
section .data
TOKEN_TYPE_IF: dd 11
section .text
section .data
TOKEN_TYPE_ELSE: dd 12
section .text
section .data
TOKEN_TYPE_RETURN: dd 13
section .text
section .data
TOKEN_TYPE_TRUE: dd 14
section .text
section .data
TOKEN_TYPE_FALSE: dd 15
section .text
section .data
TOKEN_TYPE_FOR: dd 16
section .text
section .data
TOKEN_TYPE_AT: dd 17
section .text
section .data
TOKEN_TYPE_PLUS: dd 18
section .text
section .data
TOKEN_TYPE_MINUS: dd 19
section .text
section .data
TOKEN_TYPE_STAR: dd 20
section .text
section .data
TOKEN_TYPE_SLASH: dd 21
section .text
section .data
TOKEN_TYPE_PERCENT: dd 22
section .text
section .data
TOKEN_TYPE_EQUAL: dd 23
section .text
section .data
TOKEN_TYPE_EQUAL_EQUAL: dd 24
section .text
section .data
TOKEN_TYPE_BANG: dd 25
section .text
section .data
TOKEN_TYPE_BANG_EQUAL: dd 26
section .text
section .data
TOKEN_TYPE_GREATER: dd 27
section .text
section .data
TOKEN_TYPE_GREATER_EQUAL: dd 28
section .text
section .data
TOKEN_TYPE_LESS: dd 29
section .text
section .data
TOKEN_TYPE_LESS_EQUAL: dd 30
section .text
section .data
TOKEN_TYPE_ARROW: dd 31
section .text
section .data
TOKEN_TYPE_DOUBLE_COLON: dd 32
section .text
section .data
TOKEN_TYPE_AMPERSAND: dd 33
section .text
section .data
TOKEN_TYPE_AND: dd 34
section .text
section .data
TOKEN_TYPE_OR: dd 35
section .text
section .data
TOKEN_TYPE_AS: dd 36
section .text
section .data
TOKEN_TYPE_LEFT_PAREN: dd 37
section .text
section .data
TOKEN_TYPE_RIGHT_PAREN: dd 38
section .text
section .data
TOKEN_TYPE_LEFT_BRACE: dd 39
section .text
section .data
TOKEN_TYPE_RIGHT_BRACE: dd 40
section .text
section .data
TOKEN_TYPE_LEFT_BRACKET: dd 41
section .text
section .data
TOKEN_TYPE_RIGHT_BRACKET: dd 42
section .text
section .data
TOKEN_TYPE_COMMA: dd 43
section .text
section .data
TOKEN_TYPE_SEMICOLON: dd 44
section .text
section .data
TOKEN_TYPE_PERIOD: dd 45
section .text
section .data
TOKEN_TYPE_COLON: dd 46
section .text
section .data
TOKEN_TYPE_SINGLE_QUOTE: dd 47
section .text
section .data
TOKEN_TYPE_DOUBLE_QUOTE: dd 48
section .text
section .data
TOKEN_TYPE_NEWLINE: dd 49
section .text
section .data
TOKEN_TYPE_EOF: dd 50
section .text
section .data
TOKEN_TYPE_IDENTIFIER: dd 51
section .text
section .data
TOKEN_TYPE_VOID: dd 52
section .text
section .data
TOKEN_TYPE_CHAR: dd 53
section .text
section .data
TOKEN_TYPE_FLOAT: dd 54
section .text
section .data
TOKEN_TYPE_INT: dd 55
section .text
section .data
TOKEN_TYPE_LONG: dd 56
section .text
global main
main:
push rbp
mov rbp, rsp
sub rsp, 16
mov dword [rbp - 4], edi
mov qword [rbp - 12], rsi
xor rcx, rcx
mov ecx, dword [rbp - 4]
mov rdx, 2
cmp rcx, rdx
setne al
movzx rax, al
mov rbx, rax
cmp rbx, 0
je .else0
mov rdi, 41
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
mov byte [rax + 9], 'f'
mov byte [rax + 10], 'i'
mov byte [rax + 11], 'l'
mov byte [rax + 12], 'e'
mov byte [rax + 13], 'n'
mov byte [rax + 14], 'a'
mov byte [rax + 15], 'm'
mov byte [rax + 16], 'e'
mov byte [rax + 17], '!'
mov byte [rax + 18], 10
mov byte [rax + 19], 'T'
mov byte [rax + 20], 'r'
mov byte [rax + 21], 'y'
mov byte [rax + 22], ':'
mov byte [rax + 23], ' '
mov byte [rax + 24], 'q'
mov byte [rax + 25], 'u'
mov byte [rax + 26], 'o'
mov byte [rax + 27], 'r'
mov byte [rax + 28], ' '
mov byte [rax + 29], '<'
mov byte [rax + 30], 'f'
mov byte [rax + 31], 'i'
mov byte [rax + 32], 'l'
mov byte [rax + 33], 'e'
mov byte [rax + 34], 'n'
mov byte [rax + 35], 'a'
mov byte [rax + 36], 'm'
mov byte [rax + 37], 'e'
mov byte [rax + 38], '>'
mov byte [rax + 39], 10
mov byte [rax + 40], 0
mov r12, rax
mov rdi, r12
call print_str
mov r8, rax
mov r9, 1
mov rdi, r9
call exit
mov r10, rax
.else0:
xor r11, r11
mov r11, qword [rbp - 12]
mov r13d, 1
imul r13, 8
add r11, r13
mov qword [rbp - 12], r11
xor r14, r14
mov r14, qword [rbp - 12]
sub rsp, 8
mov r14, [r14]
mov qword [rbp - 24], r14
sub rsp, 8
xor r15, r15
mov r15, qword [rbp - 24]
mov rdi, r15
call strlen
mov rcx, rax
mov qword [rbp - 32], rcx
xor rdx, rdx
mov rdx, qword [rbp - 24]
xor rbx, rbx
mov ebx, dword [rbp - 32]
mov r12, 1
sub rbx, r12
mov rdi, rdx
mov rsi, rbx
call char_at
mov r8, rax
mov r9, 'u'
cmp r8, r9
setne al
movzx rax, al
mov r10, rax
xor r13, r13
mov r13, qword [rbp - 24]
xor r11, r11
mov r11d, dword [rbp - 32]
mov r14, 2
sub r11, r14
mov rdi, r13
mov rsi, r11
call char_at
mov r15, rax
mov rcx, 'q'
cmp r15, rcx
setne al
movzx rax, al
mov r12, rax
cmp r10, 1
je .or_end_1
cmp r12, 1
je .or_end_1
mov rax, 0
jmp .or_done_2
.or_end_1:
mov rax, 1
.or_done_2:
mov rdx, rax
xor rbx, rbx
mov rbx, qword [rbp - 24]
xor r8, r8
mov r8d, dword [rbp - 32]
mov r9, 3
sub r8, r9
mov rdi, rbx
mov rsi, r8
call char_at
mov r14, rax
mov r13, '.'
cmp r14, r13
setne al
movzx rax, al
mov r11, rax
cmp rdx, 1
je .or_end_2
cmp r11, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov r15, rax
xor rcx, rcx
mov rcx, qword [rbp - 24]
xor r10, r10
mov r10d, dword [rbp - 32]
mov r12, 4
sub r10, r12
mov rdi, rcx
mov rsi, r10
call char_at
mov r9, rax
mov rdi, r9
call is_alphabetic
mov rbx, rax
mov r8, 1
cmp rbx, r8
setne al
movzx rax, al
mov r14, rax
cmp r15, 1
je .or_end_3
cmp r14, 1
je .or_end_3
mov rax, 0
jmp .or_done_4
.or_end_3:
mov rax, 1
.or_done_4:
mov r13, rax
cmp r13, 0
je .else4
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
mov r12, rax
mov rdi, r12
call print_str
mov rdx, rax
xor r11, r11
mov r11, qword [rbp - 24]
mov rdi, r11
call print_str
mov rcx, rax
mov r10, 10
mov rdi, r10
call print_char
mov r9, rax
mov rbx, 1
mov rdi, rbx
call exit
mov r8, rax
.else4:
sub rsp, 8
xor r15, r15
mov r15, qword [rbp - 24]
mov rdi, r15
call read_file
mov r14, rax
mov qword [rbp - 40], r14
xor r13, r13
mov r13, qword [rbp - 40]
mov rdi, r13
call print_str
mov r12, rax
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
xor r11, r11
mov r11, qword [rbp - 8]
xor rcx, rcx
mov rcx, qword [rbp - 8]
xor r10, r10
mov r10d, dword [rbp - 12]
add rcx, r10
mov qword [rbp - 24], rcx
xor r9, r9
mov r9, qword [rbp - 24]
mov bl, byte [r9]
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
xor r8, r8
mov r8b, byte [rbp - 1]
mov r15, 'a'
cmp r8, r15
setge al
movzx rax, al
mov r14, rax
xor r13, r13
mov r13b, byte [rbp - 1]
mov r12, 'z'
cmp r13, r12
setle al
movzx rax, al
mov rdx, rax
cmp r14, 1
jne .and_end_5
cmp rdx, 1
jne .and_end_5
mov rax, 1
jmp .and_done_6
.and_end_5:
mov rax, 0
.and_done_6:
mov r10, rax
cmp r10, 0
je .else6
mov rcx, 1
xor rax, rax
mov rax, rcx
jmp .Lret_is_alphabetic
.else6:
xor r9, r9
mov r9b, byte [rbp - 1]
mov rbx, 'A'
cmp r9, rbx
setge al
movzx rax, al
mov r8, rax
xor r15, r15
mov r15b, byte [rbp - 1]
mov r13, 'Z'
cmp r15, r13
setle al
movzx rax, al
mov r12, rax
cmp r8, 1
jne .and_end_7
cmp r12, 1
jne .and_end_7
mov rax, 1
jmp .and_done_8
.and_end_7:
mov rax, 0
.and_done_8:
mov r14, rax
cmp r14, 0
je .else8
mov rdx, 1
xor rax, rax
mov rax, rdx
jmp .Lret_is_alphabetic
.else8:
mov r10, 0
xor rax, rax
mov rax, r10
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
xor rcx, rcx
mov cl, byte [rbp - 1]
mov rdi, rcx
call is_alphabetic
mov r9, rax
xor rbx, rbx
mov bl, byte [rbp - 1]
mov r15, '0'
cmp rbx, r15
setg al
movzx rax, al
mov r13, rax
xor r8, r8
mov r8b, byte [rbp - 1]
mov r12, '9'
cmp r8, r12
setl al
movzx rax, al
mov r14, rax
cmp r13, 1
jne .and_end_9
cmp r14, 1
jne .and_end_9
mov rax, 1
jmp .and_done_10
.and_end_9:
mov rax, 0
.and_done_10:
mov rdx, rax
cmp r9, 1
je .or_end_10
cmp rdx, 1
je .or_end_10
mov rax, 0
jmp .or_done_11
.or_end_10:
mov rax, 1
.or_done_11:
mov r10, rax
cmp r10, 0
je .else11
mov rcx, 1
xor rax, rax
mov rax, rcx
jmp .Lret_is_alphanumeric
.else11:
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
xor r15, r15
mov r15, qword [rbp - 8]
mov rdi, r15
call strlen
mov r8, rax
mov qword [rbp - 24], r8
sub rsp, 8
xor r12, r12
mov r12, qword [rbp - 16]
mov rdi, r12
call strlen
mov r13, rax
mov qword [rbp - 32], r13
xor r14, r14
mov r14d, dword [rbp - 24]
xor r9, r9
mov r9d, dword [rbp - 32]
add r14, r9
mov rdx, 1
add r14, rdx
mov rdi, r14
call malloc
mov r10, rax
sub rsp, 8
mov qword [rbp - 40], r10
sub rsp, 8
xor rcx, rcx
mov rcx, qword [rbp - 40]
mov qword [rbp - 48], rcx
sub rsp, 8
mov rbx, 0
mov dword [rbp - 56], ebx
.while_start_12:
xor r15, r15
mov r15d, dword [rbp - 56]
xor r8, r8
mov r8d, dword [rbp - 24]
cmp r15, r8
setl al
movzx rax, al
mov r12, rax
cmp r12, 1
jne .while_end_12
xor r13, r13
mov r13, qword [rbp - 40]
xor r9, r9
mov r9, qword [rbp - 8]
mov dl, byte [r9]
mov byte [r13], dl
xor r14, r14
mov r14, qword [rbp - 40]
mov r10d, 1
add r14, r10
mov qword [rbp - 40], r14
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov ebx, 1
add rcx, rbx
mov qword [rbp - 8], rcx
xor r15, r15
mov r15d, dword [rbp - 56]
mov r8, 1
add r15, r8
mov qword [rbp - 56], r15
jmp .while_start_12
.while_end_12:
mov dword [rbp - 56], 0
.while_start_13:
xor r12, r12
mov r12d, dword [rbp - 56]
xor r9, r9
mov r9d, dword [rbp - 32]
cmp r12, r9
setl al
movzx rax, al
mov r13, rax
cmp r13, 1
jne .while_end_13
xor rdx, rdx
mov rdx, qword [rbp - 40]
xor r10, r10
mov r10, qword [rbp - 16]
mov r14b, byte [r10]
mov byte [rdx], r14b
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov ecx, 1
add rbx, rcx
mov qword [rbp - 40], rbx
xor r8, r8
mov r8, qword [rbp - 16]
mov r15d, 1
add r8, r15
mov qword [rbp - 16], r8
xor r12, r12
mov r12d, dword [rbp - 56]
mov r9, 1
add r12, r9
mov qword [rbp - 56], r12
jmp .while_start_13
.while_end_13:
xor r13, r13
mov r13, qword [rbp - 40]
mov r10, 0
mov byte [r13], r10b
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
xor r14, r14
mov r14, qword [rbp - 8]
mov rdi, r14
call strlen
mov rcx, rax
mov rbx, 1
add rcx, rbx
mov rdi, rcx
call malloc
mov r15, rax
sub rsp, 8
mov qword [rbp - 24], r15
xor r8, r8
mov r8, qword [rbp - 24]
xor rax, rax
mov rax, r8
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
xor r9, r9
mov r9, qword [rbp - 8]
mov rdi, r9
call strlen
mov r12, rax
mov qword [rbp - 24], r12
sub rsp, 8
mov r13, 0
mov dword [rbp - 32], r13d
.while_start_14:
xor r10, r10
mov r10d, dword [rbp - 32]
xor rdx, rdx
mov edx, dword [rbp - 24]
cmp r10, rdx
setl al
movzx rax, al
mov r14, rax
cmp r14, 1
jne .while_end_14
xor rbx, rbx
mov rbx, qword [rbp - 8]
xor rcx, rcx
mov ecx, dword [rbp - 32]
mov rdi, rbx
mov rsi, rcx
call char_at
mov r15, rax
xor r8, r8
mov r8b, byte [rbp - 9]
cmp r15, r8
sete al
movzx rax, al
mov r9, rax
cmp r9, 0
je .else15
mov r12, 1
xor rax, rax
mov rax, r12
jmp .Lret_contains_char
.else15:
xor r13, r13
mov r13d, dword [rbp - 32]
mov r10, 1
add r13, r10
mov qword [rbp - 32], r13
jmp .while_start_14
.while_end_14:
mov rdx, 0
xor rax, rax
mov rax, rdx
jmp .Lret_contains_char
.Lret_contains_char:
mov rsp, rbp
pop rbp
ret
global TOKEN_NEW
TOKEN_NEW:
push rbp
mov rbp, rsp
sub rsp, 32
mov dword [rbp - 4], edi
mov qword [rbp - 12], rsi
mov qword [rbp - 20], rdx
xor r14, r14
mov r14d, dword [rbp - 4]
xor rbx, rbx
mov rbx, qword [rbp - 12]
mov rcx, qword [rbx]
xor r15, r15
mov r15, qword [rbp - 20]
mov r8, qword [r15]
mov rdi, r14
mov rsi, rcx
mov rdx, r8
; defining TOKEN
sub rsp, 48
mov dword [rsp + 0], edi
mov qword [rsp + 4], rsi
mov qword [rsp + 20], rdx
; end TOKEN
lea r9, [rsp]
xor rax, rax
mov rax, r9
jmp .Lret_TOKEN_NEW
.Lret_TOKEN_NEW:
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
