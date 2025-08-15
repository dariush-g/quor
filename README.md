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
fn main() -> int {
    var x: int = 5;
    var p: int* = &x;
    var y: int = *p + 3;

    if (y > 5) {
        return y;
    } else {
        return 0;
    }
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
