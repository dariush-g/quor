#

```quor

@import::<io>;


def add(a: int, b: int) -> int {
    return a + b;
} 

def main() :: int {
    var n: int = Math::add(1, 2);

    print_int(n);

    return 0;
}

```
