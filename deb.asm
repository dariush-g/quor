
call Point.new
sub rsp, 8
mov qword [rbp - 8], rax
mov rax, 1
sub rsp, 8
mov r9, qword [rbp - 8]
mov dword [r9], eax
mov r10, qword [r9]
mov rdi , r10
call print_int


sub rsp, 8
mov qword [rbp - 8], rax
mov rax, 1
mov r8, qword [rbp - 8 - 0]
mov dword [r8], eax
sub rsp, 8
mov r10, qword [rbp - 8]
mov eax, dword [r10 + 0]
mov qword [rbp - 16], r9
mov r11, qword [rbp - 16]
mov rdi , r11