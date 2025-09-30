section .bss

ts_sec resq 1

section .rodata

section .text
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
sub rsp, 8
call timei
add rsp, 8
mov rcx, rax
mov qword [rbp - 8], rcx
sub rsp, 8
mov rdx, 1
mov dword [rbp - 16], edx
.while_start_0:
mov rbx, 1
cmp rbx, 1
jne .while_end_0
call timei
mov r8, rax
sub rsp, 8
mov qword [rsp], r8
xor r9, r9
mov r9d, dword [rbp - 8]
mov r10, qword [rsp]
add rsp, 8
sub r10d, r9d
sub rsp, 8
mov qword [rsp], r10
xor r11, r11
mov r11d, dword [rbp - 16]
mov r12, qword [rsp]
add rsp, 8
cmp r12, r11
sete al
movzx rax, al
mov r13, rax
cmp r13, 0
je .else1
call timei
mov r14, rax
sub rsp, 8
mov qword [rsp], r14
xor r15, r15
mov r15d, dword [rbp - 8]
mov rcx, qword [rsp]
add rsp, 8
sub ecx, r15d
mov rdi, rcx
call print_int
mov rdx, rax
xor rbx, rbx
mov ebx, dword [rbp - 16]
sub rsp, 8
mov qword [rsp], rbx
mov r8, 1
mov r9, qword [rsp]
add rsp, 8
add r9d, r8d
mov qword [rbp - 16], r9
.else1:
jmp .while_start_0
.while_end_0:
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
global timei
timei:
push rbp
mov rbp, rsp
sub rsp, 0

mov rax ,228;syscall : clock_gettime
mov rdi ,0;CLOCK_REALTIME = 0
lea rsi ,[ ts_sec]
syscall
mov rax ,[ ts_sec];put time into   reg
.Lret_timei:
mov rsp, rbp
pop rbp
ret
global factorial
factorial:
push rbp
mov rbp, rsp
sub rsp, 16
mov dword [rbp - 4], edi
xor r12, r12
mov r12d, dword [rbp - 4]
sub rsp, 8
mov qword [rsp], r12
mov r11, 1
mov r13, qword [rsp]
add rsp, 8
cmp r13, r11
sete al
movzx rax, al
mov r14, rax
sub rsp, 8
mov qword [rsp], r14
xor r15, r15
mov r15d, dword [rbp - 4]
sub rsp, 8
mov qword [rsp], r15
mov rcx, 0
mov rdx, qword [rsp]
add rsp, 8
cmp rdx, rcx
sete al
movzx rax, al
mov rbx, rax
mov r8, qword [rsp]
add rsp, 8
cmp r8, 1
je .or_end_2
cmp rbx, 1
je .or_end_2
mov rax, 0
jmp .or_done_3
.or_end_2:
mov rax, 1
.or_done_3:
mov r9, rax
cmp r9, 0
je .else3
mov r10, 1
xor rax, rax
mov rax, r10
jmp .Lret_factorial
.else3:
xor r12, r12
mov r12d, dword [rbp - 4]
sub rsp, 8
mov qword [rsp], r12
xor r13, r13
mov r13d, dword [rbp - 4]
sub rsp, 8
mov qword [rsp], r13
mov r11, 1
mov r14, qword [rsp]
add rsp, 8
sub r14d, r11d
mov rdi, r14
sub rsp, 8
call factorial
add rsp, 8
mov r15, rax
mov rdx, qword [rsp]
add rsp, 8
imul edx, r15d
xor rax, rax
mov rax, rdx
jmp .Lret_factorial
.Lret_factorial:
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
