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
mov rdi, 7
call malloc
mov byte [rax + 0], 't'
mov byte [rax + 1], 'e'
mov byte [rax + 2], 's'
mov byte [rax + 3], 't'
mov byte [rax + 4], '.'
mov byte [rax + 5], 'o'
mov byte [rax + 6], 0
mov rbx, rax
mov rdi, rbx
call file_size
mov qword [rbp - 8], rax
mov rcx, qword [rbp - 8]
mov rdi, rcx
call print_long
mov rdx, 0
mov rax, rdx
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
extern printf, strlen, fopen, fclose, fread, fwrite, fseek, ftell, rewind
section .data
fmt_int: db "%d",0
fmt_char: db "%c",0
fmt_str: db "%s",0
fmt_float: db "%f",0
fmt_long: db "%ld",0
str_true: db "true",0
str_false: db "false",0
mode_write: db "w",0
mode_read: db "rb",0
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
global read_file
read_file:
    push rbp
    mov rbp, rsp
    sub rsp, 32

    add rsp, 32
    pop rbp
    ret


global file_size
file_size:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    push rbx
    
    ; rdi already contains the filename parameter
    lea rsi, [rel mode_read]    ; Second parameter: mode
    call fopen            ; fopen(filename, "rb")
    test rax, rax
    jz .error
    mov rbx, rax
    
    ; fseek(file, 0, SEEK_END)
    mov rdi, rbx          ; FILE* stream
    xor rsi, rsi          ; offset = 0  
    mov rdx, 2            ; SEEK_END = 2
    call fseek
    
    ; ftell(file) 
    mov rdi, rbx
    call ftell
    mov rcx, rax          ; save size
    
    ; fclose(file)
    mov rdi, rbx  
    call fclose
    mov rax, rcx          ; return size
    jmp .done
    
.error:
    mov rax, -1
.done:
    pop rbx
    add rsp, 16
    pop rbp
    ret
extern malloc
extern free
extern exit
