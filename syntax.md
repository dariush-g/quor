#

```quor

@import("io")


class Math 
fn add(a: int, b: int) -> int {
    return a + b;
} 

fn main()->int {
    var n: int = Math.add(1, 2);

    print_int(n);

    return 0;
}

```
