#!/usr/bin/env bash
set -e # Stop immediately if any compilation step fails

echo "1. Building your compiler in release mode..."
cargo build --release --manifest-path ../../quorc/Cargo.toml

echo "2. Compiling benchmark programs..."
# Compile your language (adjust the CLI flags if your compiler uses different ones)
../../quorc/target/release/quorc quor_add_loop.qu

# Compile C++ with full optimizations
g++ -O3 -march=native cpp_add_loop.cpp -o cpp_add_loop

# Compile Rust with full optimizations
rustc -O -C target-cpu=native rust_add_loop.rs -o rust_add_loop

echo "3. Running benchmarks..."
# --ignore-failure is critical because returning `a` produces a non-zero exit code!
hyperfine --warmup 5 --ignore-failure \
  --command-name "Quor" './quor_add_loop' \
  --command-name "C++ (-O3)" './cpp_add_loop' \
  --command-name "Rust (-O)" './rust_add_loop'

# Clean up temporary binaries so they don't clutter your git repo
rm -f quor_add_loop cpp_add_loop rust_add_loop quor_add_loop.o cpp_add_loop.o rust_add_loop.o
