#!/bin/bash

# Quor Compiler Bootstrap Script
# This script builds the Quor compiler using the Rust implementation,
# then uses it to compile the Quor-written compiler to LLVM IR

set -e

echo "=== Quor Compiler Bootstrap ==="

# Step 1: Build the Rust compiler
echo "Building Rust compiler..."
cd pre
cargo build --release
cd ..

# Step 2: Use Rust compiler to compile the Quor compiler
echo "Compiling Quor compiler to LLVM IR..."
./pre/target/release/quor quorc/src/main_fixed.qu > quorc.ll 2>analysis.log

# Step 3: Compile LLVM IR to native binary
echo "Compiling LLVM IR to native binary..."
clang -o quorc_compiler quorc.ll

echo "Bootstrap complete! Quor compiler is now available as 'quorc_compiler'"
echo "Usage: ./quorc_compiler <filename.qu>"
