global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
mov rcx, 73
mov rdi, rcx
; ----- Inline stack struct: X -----
%define X_size 4
%define X.x 0

sub rsp, 16
mov dword [rbp + 0], edi
; struct X is now at [rsp..rsp+16]

mov rdx, rbp
add rdx, 0
sub rsp, 8
mov qword [rbp - 24], rdx
xor r8, r8
mov r8, qword [rbp - 24]
mov bl, byte [r8 + 0]
mov rdi, rbx
call print_char
mov r9, rax
mov rdi, 10
call print_char
mov rdi, rbx
mov r10, 0
xor rax, rax
mov rax, r10
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
xor r12, r12
mov r12, qword [rbp - 8]
xor r13, r13
mov r13d, dword [rbp - 12]
add r12, r13
mov qword [rbp - 24], r12
xor r14, r14
mov r14, qword [rbp - 24]
mov r15b, byte [r14]
xor rax, rax
mov rax, r15
jmp .Lret_char_at
.Lret_char_at:
mov rsp, rbp
pop rbp
ret
global is_alphabetical
is_alphabetical:
push rbp
mov rbp, rsp
sub rsp, 16
mov byte [rbp - 1], dil
xor rcx, rcx
mov cl, byte [rbp - 1]
sub rsp, 8
mov dword [rbp - 24], ecx
xor rdx, rdx
mov edx, dword [rbp - 24]
mov r8, 'z'
cmp rdx, r8
setle al
movzx rax, al
mov rbx, rax
xor r9, r9
mov r9d, dword [rbp - 24]
mov r10, 'a'
cmp r9, r10
setge al
movzx rax, al
mov r13, rax
cmp rbx, 1
jne .and_end_0
cmp r13, 1
jne .and_end_0
mov rax, 1
jmp .and_done_1
.and_end_0:
mov rax, 0
.and_done_1:
mov r12, rax
xor r14, r14
mov r14d, dword [rbp - 24]
mov r15, 'Z'
cmp r14, r15
setle al
movzx rax, al
mov rcx, rax
xor rdx, rdx
mov edx, dword [rbp - 24]
mov r8, 'A'
cmp rdx, r8
setge al
movzx rax, al
mov r9, rax
cmp rcx, 1
jne .and_end_1
cmp r9, 1
jne .and_end_1
mov rax, 1
jmp .and_done_2
.and_end_1:
mov rax, 0
.and_done_2:
mov r10, rax
cmp r12, 1
je .or_end_2
cmp r10, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov rbx, rax
cmp rbx, 0
je .else3
mov r13, 1
.else3:
mov r14, 0
xor rax, rax
mov rax, r14
jmp .Lret_is_alphabetical
.Lret_is_alphabetical:
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
mov rdx, rax
mov qword [rbp - 24], rdx
sub rsp, 8
xor r8, r8
mov r8, qword [rbp - 16]
mov rdi, r8
call strlen
mov rcx, rax
mov qword [rbp - 32], rcx
xor r9, r9
mov r9d, dword [rbp - 24]
xor r12, r12
mov r12d, dword [rbp - 32]
add r9, r12
mov r10, 1
add r9, r10
mov rdi, r9
call malloc
mov rbx, rax
sub rsp, 8
mov qword [rbp - 40], rbx
sub rsp, 8
xor r14, r14
mov r14, qword [rbp - 40]
mov qword [rbp - 48], r14
sub rsp, 8
mov r15, 0
mov dword [rbp - 56], r15d
.while_start_4:
xor rdx, rdx
mov edx, dword [rbp - 56]
xor r8, r8
mov r8d, dword [rbp - 24]
cmp rdx, r8
setl al
movzx rax, al
mov rcx, rax
cmp rcx, 1
jne .while_end_4
xor r12, r12
mov r12, qword [rbp - 40]
xor r10, r10
mov r10, qword [rbp - 8]
mov r9b, byte [r10]
mov byte [r12], r9b
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov r14d, 1
add rbx, r14
mov qword [rbp - 40], rbx
xor r15, r15
mov r15, qword [rbp - 8]
mov edx, 1
add r15, rdx
mov qword [rbp - 8], r15
xor r8, r8
mov r8d, dword [rbp - 56]
mov rcx, 1
add r8, rcx
mov qword [rbp - 56], r8
add rsp, 56
jmp .while_start_4
.while_end_4:
mov dword [rbp - 56], 0
.while_start_5:
xor r10, r10
mov r10d, dword [rbp - 56]
xor r12, r12
mov r12d, dword [rbp - 32]
cmp r10, r12
setl al
movzx rax, al
mov r9, rax
cmp r9, 1
jne .while_end_5
xor r14, r14
mov r14, qword [rbp - 40]
xor rbx, rbx
mov rbx, qword [rbp - 16]
mov dl, byte [rbx]
mov byte [r14], dl
xor r15, r15
mov r15, qword [rbp - 40]
mov ecx, 1
add r15, rcx
mov qword [rbp - 40], r15
xor r8, r8
mov r8, qword [rbp - 16]
mov r10d, 1
add r8, r10
mov qword [rbp - 16], r8
xor r12, r12
mov r12d, dword [rbp - 56]
mov r9, 1
add r12, r9
mov qword [rbp - 56], r12
add rsp, 56
jmp .while_start_5
.while_end_5:
xor rbx, rbx
mov rbx, qword [rbp - 40]
mov r14, 0
mov byte [rbx], r14b
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
xor rcx, rcx
mov rcx, qword [rbp - 8]
mov rdi, rcx
call strlen
mov r15, rax
mov r10, 1
add r15, r10
mov rdi, r15
call malloc
mov r8, rax
sub rsp, 8
mov qword [rbp - 24], r8
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
xor r12, r12
mov r12, qword [rbp - 8]
mov rdi, r12
call strlen
mov rbx, rax
mov qword [rbp - 24], rbx
sub rsp, 8
mov r14, 0
mov dword [rbp - 32], r14d
.while_start_6:
xor rdx, rdx
mov edx, dword [rbp - 32]
xor rcx, rcx
mov ecx, dword [rbp - 24]
cmp rdx, rcx
setl al
movzx rax, al
mov r10, rax
cmp r10, 1
jne .while_end_6
xor r15, r15
mov r15, qword [rbp - 8]
xor r8, r8
mov r8d, dword [rbp - 32]
mov rdi, r15
mov rsi, r8
call char_at
mov r9, rax
xor r12, r12
mov r12b, byte [rbp - 9]
cmp r9, r12
sete al
movzx rax, al
mov rbx, rax
cmp rbx, 0
je .else7
mov r14, 1
.else7:
xor rdx, rdx
mov edx, dword [rbp - 32]
mov rcx, 1
add rdx, rcx
mov qword [rbp - 32], rdx
add rsp, 32
jmp .while_start_6
.while_end_6:
mov r10, 0
xor rax, rax
mov rax, r10
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
