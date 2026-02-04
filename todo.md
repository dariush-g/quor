# Todo

make @define a text replacement

add conditional compilation declarations:

    @if target_arch = "aarch64"
        print("hello");
    @endif

cfg ir - add ro_data and bss inline asm

modularize x86 codegen for IR
make strings static in .data
add partial SSA to IR

start ARM codegen

add line numbers to errors and improve error warnings in general

features todo:
    type inference
    namespaces:
        @import <io.qu> | io
    generics -> monomorph / static dispatch
    traits -> dynamic dispatch
    closures