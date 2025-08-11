# Quor

## What it can do so far

- Integers and booleans
- Pointers (`&` for address-of, `*` for dereference)
- Arrays (stack allocated)
- Variables (stack-allocated for now, 8 bytes each)
- Functions and function calls
- `if` / `while` / `return`
- Basic math and comparisons

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
