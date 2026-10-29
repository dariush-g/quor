# Quor

Quor is a small, experimental systems programming language that compiles to native x86_64 and aarch64 assembly, built for pedagogical reasons.

## Architecture

Source (.qu) --> Lexer --> Parser --> AST --> MIR (block-based) --> LIR (graph coloring regalloc) --> x86_64 / aarch64

### Current Features

- Integers, characters, strings, booleans, floats
- Pointers (`&` for address, `*` for dereference)
- Arrays with indexing (No indexing for pointers yet)
- Basic structs (stack-allocated with heap option)
- Basic unions (stack-allocated with heap option)
- Variables & global constants
- Functions and function calls
- `if` / 'else' / `while`
- Imports to a standard lib and to local files
- Basic memory management using C's `malloc()` and `free()`
- Basic output using C's printf
- Variadic functions
- Inline assembly (but no variable interop within it yet)
- Native x86_64 codegen
- Type inference for variable assignments
- Custom target builds between x86_64 and aarch64 (linker and assembler still use host's architecture though)
- conditional compilation
  
- Monomorphized generic structs (AI-assisted implementation)
- Native AArch64 codegen (AI-assisted implementation)
- Macro in debug flag (AI-assisted implementation)

## Examples

### Functions

```python
def add(a: int, b: int) :: int {
    return a + b;
}

def main() :: int {

	let result = add(1, 2);
    print("%d", result);

    return 0;
}
```

### Generic Structs

```python
struct Example<T, K> {
    x: T;
    y: K;
}

def example() :: Example<int, char*> {
    return Example {
        x: 10,
        y: "hello world"
    };
}
```

### Loops

```python
def main() :: int {
    let i: int = 0;

    for (i < 10 :: i++) {
        // do something
    }

    return 0;
}
```

### Structs

```python
@import <io.qu>

struct Person {
    name: char*;
    age: int;
}

def get_name(self: Person*) :: char* {
    return self.name;
}

struct Example {
    x: int;
}

def main() :: int {
    let example = malloc(sizeof(Example)) as Example*;

    example.x = 42;

    print("%d", example.x);

	free(example);

	let person = Person { name: "bob", age: 10 };

    return 0;
}
```

### Inline assembly

```python
@trust_ret
def get_time_int() :: int {
    @__asm__ {
        mov rax, 228
        mov rdi, 0
        lea rsi, [ts_sec]
        syscall
        mov rax, [ts_sec]
    }
}

```

### Global constants

```python
@import <io.qu>
@const ONE_HUNDRED = 100
def main() :: int {
	print("%d", ONE_HUNDRED);
	return 0;
}
```

## Known limitations:

- aarch64 register sizes are buggy
- 6 parameter maximum for function calls
- No bitwise operations yet
- No pointer indexing yet

Flags 

    --debug              stage timings
    --emit-tokens        token stream
    --emit-ast           AST
    --emit-typed         typed AST
    --emit-mir           MIR
    --emit-asm           generated assembly
    --target-arch=<a>    x86_64 | aarch64  (codegen only)
    --target-os=<os>     linux | macos     (codegen only)
	--help 				

Add `@keep_asm` anywhere in the file to keep intermediate
assembly.

