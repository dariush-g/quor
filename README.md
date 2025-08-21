# Quor

## Current Features

- Integers, chars, and booleans
- Pointers (`&` for address-of, `*` for dereference)
- Arrays
- Basic classes (heap-allocated) with functions
- Variables (stack-allocated for now, 8 bytes each)
- Functions and function calls
- `if` / `while` / `return`

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
struct Guy {
    name: string;
    age: int;
}

def get_name(self: Guy*) :: string {
    return self.name;
}

```
