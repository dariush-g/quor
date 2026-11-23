# Benchmarks: Quor vs. C++ and Rust

**Quor** against **C++ (`clang++ -O3`)** and **Rust (`rustc -O`)** on an identical CPU-bound workload: calling an `add(x, y)` function inside a loop **100,000,000 times**.

## Test Workload

```c
int add(int x, int y) {
    return x + y;
}

int main() {
    int a = 0;
    for (int i = 0; i < 100000000; i++) {
        a = add(a, i);
    }
    return a;
}
```

## Results

Benchmarked using `hyperfine` (`--warmup 5`) on macOS M1:

| Language / Compiler | Mean Wall Time | Std Dev ($\sigma$) | Relative Speed | Peak Memory |
| :--- | :--- | :--- | :--- | :--- |
| **C++ (`-O3`)** | **1.5 ms** | $\pm$ 0.7 ms | **105.6x faster** | 1.6 MiB |
| **Rust (`-O`)** | **2.0 ms** | $\pm$ 0.2 ms | **82.0x faster** | 1.7 MiB |
| **Quor** | **163.0 ms** | $\pm$ 5.8 ms | Baseline (1.0x) | 1.6 MiB |


When increasing the loop count by 100x (from 1 million to 100 million iterations):
* **Quor** scaled linearly (~1.8 ms $\rightarrow$ 163.0 ms), executing every loop iteration and function call in $O(n)$ time.
* **C++ and Rust** stayed completely flat (~1.5–2.0 ms, equivalent to macOS process startup overhead).