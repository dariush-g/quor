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
mov r8, 1
mov rdi, r8
call exit
mov r9, rax
.else0:
xor r10, r10
mov r10, qword [rbp - 12]
mov r11d, 1
imul r11, 8
add r10, r11
mov qword [rbp - 12], r10
xor r12, r12
mov r12, qword [rbp - 12]
sub rsp, 8
mov r12, [r12]
mov qword [rbp - 24], r12
sub rsp, 8
xor r13, r13
mov r13, qword [rbp - 24]
mov rdi, r13
call strlen
mov r14, rax
mov qword [rbp - 32], r14
xor r15, r15
mov r15, qword [rbp - 24]
xor rcx, rcx
mov ecx, dword [rbp - 32]
mov rdx, 1
sub rcx, rdx
mov rdi, r15
mov rsi, rcx
call char_at
mov rbx, rax
mov r8, 'u'
cmp rbx, r8
setne al
movzx rax, al
mov r9, rax
xor r11, r11
mov r11, qword [rbp - 24]
xor r10, r10
mov r10d, dword [rbp - 32]
mov r12, 2
sub r10, r12
mov rdi, r11
mov rsi, r10
call char_at
mov r13, rax
mov r14, 'q'
cmp r13, r14
setne al
movzx rax, al
mov rdx, rax
cmp r9, 1
jne .and_end_1
cmp rdx, 1
jne .and_end_1
mov rax, 1
jmp .and_done_2
.and_end_1:
mov rax, 0
.and_done_2:
mov r15, rax
xor rcx, rcx
mov rcx, qword [rbp - 24]
xor rbx, rbx
mov ebx, dword [rbp - 32]
mov r8, 3
sub rbx, r8
mov rdi, rcx
mov rsi, rbx
call char_at
mov r12, rax
mov r11, '.'
cmp r12, r11
setne al
movzx rax, al
mov r10, rax
cmp r15, 1
jne .and_end_2
cmp r10, 1
jne .and_end_2
mov rax, 1
jmp .and_done_3
.and_end_2:
mov rax, 0
.and_done_3:
mov r13, rax
cmp r13, 0
je .else3
mov r14, 1
mov rdi, r14
call exit
mov r9, rax
.else3:
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
xor rcx, rcx
mov rcx, qword [rbp - 8]
xor rbx, rbx
mov ebx, dword [rbp - 12]
add rcx, rbx
mov qword [rbp - 24], rcx
xor r12, r12
mov r12, qword [rbp - 24]
mov r11b, byte [r12]
xor rax, rax
mov rax, r11
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
xor r15, r15
mov r15, qword [rbp - 8]
mov rdi, r15
call strlen
mov r10, rax
mov qword [rbp - 24], r10
sub rsp, 8
xor r13, r13
mov r13, qword [rbp - 16]
mov rdi, r13
call strlen
mov r14, rax
mov qword [rbp - 32], r14
xor r9, r9
mov r9d, dword [rbp - 24]
xor rdx, rdx
mov edx, dword [rbp - 32]
add r9, rdx
mov rbx, 1
add r9, rbx
mov rdi, r9
call malloc
mov rcx, rax
sub rsp, 8
mov qword [rbp - 40], rcx
sub rsp, 8
xor r12, r12
mov r12, qword [rbp - 40]
mov qword [rbp - 48], r12
sub rsp, 8
mov r11, 0
mov dword [rbp - 56], r11d
.while_start_4:
xor r15, r15
mov r15d, dword [rbp - 56]
xor r10, r10
mov r10d, dword [rbp - 24]
cmp r15, r10
setl al
movzx rax, al
mov r13, rax
cmp r13, 1
jne .while_end_4
xor r14, r14
mov r14, qword [rbp - 40]
xor rdx, rdx
mov rdx, qword [rbp - 8]
mov bl, byte [rdx]
mov byte [r14], bl
xor r9, r9
mov r9, qword [rbp - 40]
mov ecx, 1
add r9, rcx
mov qword [rbp - 40], r9
xor r12, r12
mov r12, qword [rbp - 8]
mov r11d, 1
add r12, r11
mov qword [rbp - 8], r12
xor r15, r15
mov r15d, dword [rbp - 56]
mov r10, 1
add r15, r10
mov qword [rbp - 56], r15
add rsp, 56
jmp .while_start_4
.while_end_4:
mov dword [rbp - 56], 0
.while_start_5:
xor r13, r13
mov r13d, dword [rbp - 56]
xor rdx, rdx
mov edx, dword [rbp - 32]
cmp r13, rdx
setl al
movzx rax, al
mov r14, rax
cmp r14, 1
jne .while_end_5
xor rbx, rbx
mov rbx, qword [rbp - 40]
xor rcx, rcx
mov rcx, qword [rbp - 16]
mov r9b, byte [rcx]
mov byte [rbx], r9b
xor r11, r11
mov r11, qword [rbp - 40]
mov r12d, 1
add r11, r12
mov qword [rbp - 40], r11
xor r10, r10
mov r10, qword [rbp - 16]
mov r15d, 1
add r10, r15
mov qword [rbp - 16], r10
xor r13, r13
mov r13d, dword [rbp - 56]
mov rdx, 1
add r13, rdx
mov qword [rbp - 56], r13
add rsp, 56
jmp .while_start_5
.while_end_5:
xor r14, r14
mov r14, qword [rbp - 40]
mov rcx, 0
mov byte [r14], cl
xor rbx, rbx
mov rbx, qword [rbp - 48]
xor rax, rax
mov rax, rbx
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
mov r12, rax
mov r11, 1
add r12, r11
mov rdi, r12
call malloc
mov r15, rax
sub rsp, 8
mov qword [rbp - 24], r15
xor r10, r10
mov r10, qword [rbp - 24]
xor rax, rax
mov rax, r10
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
xor rdx, rdx
mov rdx, qword [rbp - 8]
mov rdi, rdx
call strlen
mov r13, rax
mov qword [rbp - 24], r13
sub rsp, 8
mov r14, 0
mov dword [rbp - 32], r14d
.while_start_6:
xor rcx, rcx
mov ecx, dword [rbp - 32]
xor rbx, rbx
mov ebx, dword [rbp - 24]
cmp rcx, rbx
setl al
movzx rax, al
mov r9, rax
cmp r9, 1
jne .while_end_6
xor r11, r11
mov r11, qword [rbp - 8]
xor r12, r12
mov r12d, dword [rbp - 32]
mov rdi, r11
mov rsi, r12
call char_at
mov r15, rax
xor r10, r10
mov r10b, byte [rbp - 9]
cmp r15, r10
sete al
movzx rax, al
mov rdx, rax
cmp rdx, 0
je .else7
mov r13, 1
.else7:
xor r14, r14
mov r14d, dword [rbp - 32]
mov rcx, 1
add r14, rcx
mov qword [rbp - 32], r14
add rsp, 32
jmp .while_start_6
.while_end_6:
mov rbx, 0
xor rax, rax
mov rax, rbx
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
