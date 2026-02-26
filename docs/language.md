# Language Features/Syntax
## At declarations:

### Function attributes
@trust_ret -- removes checking for return statements
@variadic -- allows for an undeclared number of parameters (on macos, variadic args are passed by stack)
@no_frame -- does not add function prologue or epilogue in codegen

### Inline assembly
@asm -- inline assembly
@asm_bss -- inline assembly in bss section
@asm_ro -- inline assembly in rodata section
@asm_data -- inline assembly in data section

### Imports
@import <filename.qu> -- imports filename.qu from the std lib
@import "filename.qu" -- imports filename.qu locally
@extern module -- externs said module

@keep_asm -- keeps the compiled assembly file (thinking of changing this to a compile flag)

### Data
@union -- precedes the __struct__ keyword in order to make it a union type
@const -- creates a global constant 

### Conditional compilation
@cfg[CONDITION] {} -- e.g. @cfg[target_os = "macos"] {}
