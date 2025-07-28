// section .text
// global print_string

// ; void print_string(char *str);
// ; rdi = pointer to null-terminated string
// print_string:
//     push rbp
//     mov rbp, rsp

//     ; Calculate string length
//     mov rsi, rdi      ; rsi = pointer to string
//     xor rcx, rcx      ; counter = 0

// .length_loop:
//     mov al, byte [rsi + rcx]
//     cmp al, 0
//     je .length_done
//     inc rcx
//     jmp .length_loop

// .length_done:
//     ; rcx = length of string

//     ; Prepare syscall parameters
//     mov rax, 1        ; sys_write
//     mov rdi, 1        ; stdout fd = 1
//     mov rsi, rsi      ; pointer to string (already in rsi)
//     mov rdx, rcx      ; length
//     syscall

//     ; Restore stack frame and return
//     mov rsp, rbp
//     pop rbp
//     ret

pub fn add_print() -> String {
    format!(
        r#"
global print_string

; void print_string(char *str);
; rdi = pointer to null-terminated string
print_string:
    push rbp
    mov rbp, rsp

    ; Calculate string length
    mov rsi, rdi      ; rsi = pointer to string
    xor rcx, rcx      ; counter = 0

.length_loop:
    mov al, byte [rsi + rcx]
    cmp al, 0
    je .length_done
    inc rcx
    jmp .length_loop

.length_done:
    ; rcx = length of string

    ; Prepare syscall parameters
    mov rax, 1        ; sys_write
    mov rdi, 1        ; stdout fd = 1
    mov rsi, rsi      ; pointer to string (already in rsi)
    mov rdx, rcx      ; length
    syscall

    ; Restore stack frame and return
    mov rsp, rbp
    pop rbp
    ret
"#
    )
}

// section .text
// global _start

// _start:
//     ; Allocate 14 bytes on stack (13 chars + null terminator)
//     sub rsp, 16      ; align to 16 bytes for safety

//     mov byte [rsp], 'H'
//     mov byte [rsp+1], 'e'
//     mov byte [rsp+2], 'l'
//     mov byte [rsp+3], 'l'
//     mov byte [rsp+4], 'o'
//     mov byte [rsp+5], ','
//     mov byte [rsp+6], ' '
//     mov byte [rsp+7], 'w'
//     mov byte [rsp+8], 'o'
//     mov byte [rsp+9], 'r'
//     mov byte [rsp+10], 'l'
//     mov byte [rsp+11], 'd'
//     mov byte [rsp+12], '!'
//     mov byte [rsp+13], 0       ; null terminator

//     ; Pass pointer to string (rsp) to print function
//     mov rdi, rsp
//     call print_string

//     add rsp, 16     ; restore stack

//     ; exit syscall
//     mov rax, 60
//     xor rdi, rdi
//     syscall

// mov eax, 1\nint 0x80

fn align16(n: usize) -> usize {
    (n + 15) & !15
}

pub fn print(str: String) -> String {
    let alloc = align16(str.len() + 2);
    let mut n = String::new();
    n.push_str(&format!("sub rsp, {alloc}\n"));

    for (i, c) in str.chars().enumerate() {
        n.push_str(&format!("mov byte [rsp + {i}], '{c}'\n"));
    }

    let newline_index = str.len();
    n.push_str(&format!("mov byte [rsp + {newline_index}], 10\n"));

    n.push_str(&format!("mov byte [rsp + {}], 0\n", str.len() + 1));

    n.push_str("mov rdi, rsp\n");
    n.push_str("call print_string\n");
    n.push_str(&format!("add rsp, {alloc}\n"));

    n
}
