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
@import::<io>;

def add(a: int, b: int) :: int {
    return a + b;
}

def main() :: int {
    
    print_int(add(1, 2));

    return 0;
}
```

```quor
class Circle {
    radius: int;

    fn area(self: Circle) -> int {
        return 3 * self.radius;
    }
}
```
