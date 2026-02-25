# Todo

make static strings maintain proper break characters when being put in rodata

variadic functions on Apple need to be loaded onto stack

make @define a text replacement

add conditional compilation declarations:

    @if target_arch = "aarch64"
        print("hello");
    @endif

cfg ir - add ro_data and bss inline asm

add line numbers to errors and improve error warnings in general

features todo:
generics -> monomorph / static dispatch
traits -> dynamic dispatch
trained inlining cost function
