# Quor

Quor is a small, experimental systems programming language with Rust-inspired syntax that compiles to native x86_64 and aarch64 assembly.

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
- `if` / 'else' / `while` / 'for'
- Imports to a standard lib and to local files
- Basic memory management using C's `malloc()` and `free()`
- Basic output using C's printf
- Variadic functions
- Inline assembly
- Multi-target codegen (x86_64, aarch64)
- Type inference for variable assignments
- Custom target builds between x86_64 and aarch64
- conditional compilation
- Monomorphized generic structs

## Examples

### Functions

```quor
def add(a: int, b: int) :: int {
    return a + b;
}

def main() :: int {

    print_int(add(1, 2));

    return 0;
}
```

### Generic Structs

```quor
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

```quor
def main() :: int {
    let i: int = 0;

    for (i < 10 :: i++) {
        // do something
    }

    return 0;
}
```

### Structs

```quor
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

```quor
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

```quor
@import <io.qu>
@const ONE_HUNDRED = 100
def main() :: int {
	print("%d", ONE_HUNDRED);
	return 0;
}
```

## Known limitations:

- 6 parameter maximum for function calls
- No bitwise operations yet
- No pointer indexing yet
- codegen backends are a work in progress
