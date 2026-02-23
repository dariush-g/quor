.extern _printf

.section __TEXT,__cstring
__q_g_0:
    .asciz "hello world\\n"

.section __TEXT,__const
.section __DATA,__data
.section __DATA,__bss
.section __TEXT,__text
__q_f_print:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_print_0:

bl _printf

b .Lret_print
.Lret_print:
ldp x29, x30, [sp], #16
ret
.globl _main
_main:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_main_1:
adrp x16, __q_g_0@PAGE
add x16, x16, __q_g_0@PAGEOFF
mov x10, x16
mov x0, x10
bl __q_f_print
mov x9, x0
mov x16, #0
mov x0, x16
b .Lret_main
.Lret_main:
ldp x29, x30, [sp], #16
ret
