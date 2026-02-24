#

```quor

def add(a: int, b: int) -> int {
    return a + b;
} 

def main() :: int {
    var n = add(1, 2);
    print("%d", n);

    @cfg[target_os = "macos"] {
        print("macos");
    }

    return 0;
}

```
