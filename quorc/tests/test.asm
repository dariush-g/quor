extern printf


section .rodata
__q_g_0:
    db "%f", 0

__q_fc_4024000000000000:
    dq 0x4024000000000000
__q_fc_3ff3333333333333:
    dq 0x3ff3333333333333

section .data
section .bss
section .text
global main
main:
push rbp
mov rbp, rsp
sub rsp, 0
.Lblock_main_1: 
movsd xmm0, qword [rel __q_fc_4024000000000000]
movsd xmm1, qword [rel __q_fc_3ff3333333333333]
movsd xmm2, xmm0
mulsd xmm2, xmm1
movsd xmm0, xmm2
lea rax, qword [rel __q_g_0]
mov rcx, rax
mov rdi, rcx
movsd xmm0, xmm0
mov eax, 1
call __q_f_print

mov rax, rax
mov rax, 0
jmp .Lret_main
.Lret_main:
mov rsp, rbp
pop rbp
ret
__q_f_print:
.Lblock_print_0: 

sub rsp ,8
call printf
add rsp ,8

ret
