#!/bin/bash

set -e

echo "=== Building Latest Release ==="
cargo build --release

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$SCRIPT_DIR/benchmarks"

mkdir -p "$BENCHMARK_DIR"
cd "$BENCHMARK_DIR"

echo "=== Cloning codebases ==="

if [ ! -d "coreutils" ]; then
  git clone https://github.com/uutils/coreutils.git
fi

if [ ! -d "Chart.js" ]; then
  git clone https://github.com/chartjs/Chart.js
fi

cd "$SCRIPT_DIR"

echo "Running benchmarks..."

echo "=== Benchmark: This Codebase ==="
hyperfine --runs 100 --warmup 3 'target/release/auto-uml --source-code src/ --lang rust --destination uml.md'

echo "=== Benchmark: CoreUtils ==="
hyperfine --runs 100 --warmup 3 "target/release/auto-uml --source-code $BENCHMARK_DIR/coreutils --lang rust --destination uml.md"

echo "=== Benchmark: Chart.js ==="
hyperfine --runs 100 --warmup 3 "target/release/auto-uml --source-code $BENCHMARK_DIR/Chart.js --lang javascript --destination uml.md"

echo "Cleaning up benchmarks..."
rm -rf "$BENCHMARK_DIR"

echo "Done!"
