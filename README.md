# Quor ℺

## Quor is a small (not completely debugged) programming language that compiles directly to x86_64 assembly.

### Current Features

- Integers, chars, strings, booleans
- Pointers (`&` for address, `*` for dereference)
- Arrays with indexing (No indexing for pointers yet)
- Basic structs (stack-allocated with heap option)
- Basic unions (stack-allocated with heap option)
- Variables
- Functions and function calls
- `if` / 'else' / `while` / 'for'
- imports to a standard lib and to local files
- Basic memory management using C's `malloc()` and `free()`
- Inline assembly

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
struct Person {
    name: string;
    age: int;
}

def get_name(self: Person*) :: string {
    return self.name;
}

struct Example {
    x: int;
}

def main() :: int {
    let example: Example* = malloc(sizeof(Example)) as Example*;

    example.x = 42;

    print_int(example.x);

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
