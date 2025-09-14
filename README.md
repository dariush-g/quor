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
- `if` / `while`
- imports to a standard lib and to local files
- Basic memory management using C's `malloc()` and `free()`

## Example

**Code:**

```quor
def add(a: int, b: int) :: int {
    return a + b;
}

def main() :: int {

    print_int(add(1, 2));

    return 0;
}
```

```quor
struct Person {
    name: string;
    age: int;
}

def get_name(self: Person*) :: string {
    return self.name;
}

```

```quor
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
