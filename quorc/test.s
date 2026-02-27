.extern _printf

.section __TEXT,__cstring
.section __TEXT,__const
.section __DATA,__data
.section __DATA,__bss
.section __TEXT,__text
.globl _main
_main:
stp x29, x30, [sp, #-16]!
mov x29, sp
sub sp, sp, #16
.Lblock_main_0:
mov x16, #10
mov x0, x16
bl __q_f_malloc
mov x9, x0
str x9, [x29, #-8]
mov x16, #0
mov x0, x16
b .Lret_main
.Lret_main:
mov sp, x29
ldp x29, x30, [sp], #16
ret
__q_f_malloc:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lret_malloc:
ldp x29, x30, [sp], #16
ret
