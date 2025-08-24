extern malloc
global _start
_start:
call main
mov rbx, rax
mov rdi, 10
call print_char
mov rdi, rbx
mov rax, 60
syscall
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
sub rsp, 8
mov rdi, 6
call malloc
mov byte [rax + 0], 'h'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 'l'
mov byte [rax + 3], 'l'
mov byte [rax + 4], 'o'
mov byte [rax + 5], 0
mov rbx, rax
mov qword [rbp - 8], rbx
mov rcx, qword [rbp - 8]
mov rdi, rcx
call print_str
mov rdi, 11
call malloc
mov byte [rax + 0], 'o'
mov byte [rax + 1], 'u'
mov byte [rax + 2], 't'
mov byte [rax + 3], 'p'
mov byte [rax + 4], 'u'
mov byte [rax + 5], 't'
mov byte [rax + 6], '.'
mov byte [rax + 7], 't'
mov byte [rax + 8], 'x'
mov byte [rax + 9], 't'
mov byte [rax + 10], 0
mov r12, rax
mov rdi, 8
call malloc
mov byte [rax + 0], 't'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 's'
mov byte [rax + 3], 't'
mov byte [rax + 4], 'i'
mov byte [rax + 5], 'n'
mov byte [rax + 6], 'g'
mov byte [rax + 7], 0
mov r13, rax
mov rdi, r12
mov rsi, r13
call write_to_file
mov rdi, 12
call malloc
mov byte [rax + 0], 'o'
mov byte [rax + 1], 'u'
mov byte [rax + 2], 't'
mov byte [rax + 3], 'p'
mov byte [rax + 4], 'u'
mov byte [rax + 5], 't'
mov byte [rax + 6], '2'
mov byte [rax + 7], '.'
mov byte [rax + 8], 't'
mov byte [rax + 9], 'x'
mov byte [rax + 10], 't'
mov byte [rax + 11], 0
mov r14, rax
mov rdi, 8
call malloc
mov byte [rax + 0], 't'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 's'
mov byte [rax + 3], 't'
mov byte [rax + 4], 'i'
mov byte [rax + 5], 'n'
mov byte [rax + 6], 'g'
mov byte [rax + 7], 0
mov r15, rax
mov rdi, r14
mov rsi, r15
call write_to_file
mov rdi, 12
call malloc
mov byte [rax + 0], 'o'
mov byte [rax + 1], 'u'
mov byte [rax + 2], 't'
mov byte [rax + 3], 'p'
mov byte [rax + 4], 'u'
mov byte [rax + 5], 't'
mov byte [rax + 6], '3'
mov byte [rax + 7], '.'
mov byte [rax + 8], 't'
mov byte [rax + 9], 'x'
mov byte [rax + 10], 't'
mov byte [rax + 11], 0
mov rbx, rax
mov rdi, 9
call malloc
mov byte [rax + 0], 't'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 's'
mov byte [rax + 3], 't'
mov byte [rax + 4], 'i'
mov byte [rax + 5], 10
mov byte [rax + 6], 'n'
mov byte [rax + 7], 'g'
mov byte [rax + 8], 0
mov r12, rax
mov rdi, rbx
mov rsi, r12
call write_to_file
mov rdx, 0
mov rax, rdx
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
extern printf, strlen, fopen, fclose, fread, fwrite
section .data
fmt_int: db "%d",0
fmt_char: db "%c",0
fmt_str: db "%s",0
fmt_float: db "%f",0
str_true: db "true",0
str_false: db "false",0
mode_write: db "w",0
section .text
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

global write_to_file
write_to_file:
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
extern malloc
extern free
extern exit
