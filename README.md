# Quor

## Current Features

- Integers, chars, strings, booleans
- Pointers (`&` for address-of, `*` for dereference)
- Arrays
- Basic structs (heap-allocated)
- Variables (stack-allocated for now, 8 bytes each)
- Functions and function calls
- `if` / `while`
- imports to a standard lib and to local files

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
