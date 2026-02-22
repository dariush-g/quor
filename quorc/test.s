.extern _printf

.section __TEXT,__cstring
__q_g_0:
    .asciz "%d"

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
__q_f_add:
stp x29, x30, [sp, #-16]!
mov x29, sp
sub sp, sp, #16
.Lblock_add_2:
mov x10, x0
mov x9, x1
add x11, x10, x9
mov x0, x11
b .Lret_add
.Lblock_add_1:
mov x16, #1
mov x0, x16
mov x16, #2
mov x1, x16
bl __q_f_add
mov x9, x0
str x9, [x29, #-8]
adrp x16, __q_g_0@PAGE
add x16, x16, __q_g_0@PAGEOFF
mov x10, x16
ldr w16, [x29, #-8]
mov x11, x16
mov x0, x10
mov x1, x11
bl __q_f_print
mov x9, x0
mov x16, #0
mov x0, x16
b .Lret_add
.Lret_add:
mov sp, x29
ldp x29, x30, [sp], #16
ret
.globl _main
_main:
stp x29, x30, [sp, #-16]!
mov x29, sp
sub sp, sp, #16
.Lblock_main_1:
mov x16, #1
mov x0, x16
mov x16, #2
mov x1, x16
bl __q_f_add
mov x9, x0
str x9, [x29, #-8]
adrp x16, __q_g_0@PAGE
add x16, x16, __q_g_0@PAGEOFF
mov x10, x16
ldr w16, [x29, #-8]
mov x9, x16
mov x0, x10
mov x1, x9
bl __q_f_print
mov x11, x0
mov x16, #0
mov x0, x16
b .Lret_main
.Lret_main:
mov sp, x29
ldp x29, x30, [sp], #16
ret
