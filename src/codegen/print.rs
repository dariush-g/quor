pub fn add_print_int(string: &mut String) {
    string.push_str(
        r#"section .text
global print_int

print_int:
    
    push rax
    push rcx
    push rdx
    push rsi
    push rbx

    mov rax, rdi           
    mov rcx, 10
    lea rsi, [rel int_buf + 20] 
    mov byte [rsi], 10    
    dec rsi

    test rax, rax
    jns .convert
    neg rax
    mov bl, '-'         
    jmp .convert

.convert:
    xor rbx, rbx     
.loop:
    xor rdx, rdx
    div rcx              
    add dl, '0'
    dec rsi
    mov [rsi], dl
    test rax, rax
    jnz .loop

    cmp bl, '-'
    jne .done
    dec rsi
    mov [rsi], bl

.done:
    mov rdx, int_buf + 21
    sub rdx, rsi          
    mov rax, 1            
    mov rdi, 1            
    mov rsi, rsi           
    syscall

    pop rbx
    pop rsi
    pop rdx
    pop rcx
    pop rax
    ret

section .bss
int_buf resb 21  ;
"#,
    );
}
