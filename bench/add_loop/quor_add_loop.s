
.section __TEXT,__cstring
.section __TEXT,__const
.section __DATA,__data
.section __DATA,__bss
.section __TEXT,__text
.globl _main
_main:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_main_1:
mov w16, #0
mov w11, w16
mov w16, #0
mov w10, w16
b .Lblock_main_2
.Lblock_main_2:
mov w17, #1000000
cmp w10, w17
cset w9, lt
cbnz w9, .Lblock_main_3
b .Lblock_main_4
.Lblock_main_3:
mov w0, w11
mov w1, w10
bl __q_f_add
mov x9, x0
mov w11, w9
mov w0, w11
b .Lret_main
.Lret_main:
ldp x29, x30, [sp], #16
ret
__q_f_add:
stp x29, x30, [sp, #-16]!
mov x29, sp
.Lblock_add_0:
mov w10, w0
mov w11, w1
add w9, w10, w11
mov w0, w9
b .Lret_add
.Lret_add:
ldp x29, x30, [sp], #16
ret
