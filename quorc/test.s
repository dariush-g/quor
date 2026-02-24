.extern _printf

.section __TEXT,__cstring
__q_g_0:
    .asciz "%d"

.section __TEXT,__const
.section __DATA,__data
.section __DATA,__bss
.section __TEXT,__text
.globl _main
_main:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_main_1:
adrp x16, __q_g_0@PAGE
add x16, x16, __q_g_0@PAGEOFF
mov x9, x16
mov x0, x9
mov x16, #1
mov x1, x16
bl __q_f_print
mov x10, x0
mov x16, #0
mov x0, x16
b .Lret_main
.Lret_main:
ldp x29, x30, [sp], #16
ret
__q_f_print:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_print_0:

str x1 ,[ sp ,# 16]
bl _printf

b .Lret_print
.Lret_print:
ldp x29, x30, [sp], #16
ret
