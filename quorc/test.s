.extern _printf

.section __TEXT,__cstring
__q_g_0:
    .asciz "%d, %s"

__q_g_1:
    .asciz "hello"

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
adrp x16, __q_g_1@PAGE
add x16, x16, __q_g_1@PAGEOFF
mov x11, x16
sub sp, sp, #16
mov x16, #20
str x16, [sp, #0]
str x11, [sp, #8]
mov x0, x9
bl __q_f_print
mov x10, x0
add sp, sp, #16
mov x16, #0
mov x0, x16
b .Lret_main
.Lret_main:
ldp x29, x30, [sp], #16
ret
__q_f_print:
.Lblock_print_0:

b _printf

