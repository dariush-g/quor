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

@import::<io>

def main() :: int {
    let x: int = 5;
    let p: int* = &x;
    let y: int = *p + 3;

    print_int(y);
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
