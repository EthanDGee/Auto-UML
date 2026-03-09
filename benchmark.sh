#!/bin/bash

set -e

RUNS=100
WARMUP=3

echo "=== Building Latest Release ==="
cargo build --release

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$SCRIPT_DIR/benchmarks"
TOOL_PATH="target/release/auto-uml"

mkdir -p "$BENCHMARK_DIR"
cd "$BENCHMARK_DIR"

echo "=== Cloning codebases ==="

if [ ! -d "coreutils" ]; then
  git clone --depth 1 https://github.com/uutils/coreutils.git
fi

if [ ! -d "Chart.js" ]; then
  git clone --depth 1 https://github.com/chartjs/Chart.js
fi

if [ ! -d "BuildCLI" ]; then
  git clone --depth 1 https://github.com/BuildCLI/BuildCLI
fi

if [ ! -d "faker-cxx" ]; then
  git clone --depth 1 https://github.com/cieslarmichal/faker-cxx
fi

if [ ! -d "ReactiveUI" ]; then
  git clone --depth 1 https://github.com/reactiveui/ReactiveUI
fi

if [ ! -d "authass" ]; then
  git clone --depth 1 https://github.com/authpass/authpass.git
fi

if [ ! -d "jupyterlab" ]; then
  git clone --depth 1 ttps://github.com/authpass/authpass.git
fi

if [ ! -d "bitwarden-server"]; then
  git clone --depth 1 https://github.com/bitwarden/server bitwarden-server
fi

if [! -d "Platypus"]; then
  git clone --depth 1 https://github.com/sveinbjornt/Platypus
fi

cd "$SCRIPT_DIR"

echo "Running benchmarks..."

echo "=== Benchmark: This Codebase ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code src/ --lang rust --destination uml.md"

echo "=== Benchmark: Chart.js ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/Chart.js --lang javascript --destination uml.md"

echo "=== Benchmark: CoreUtils ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/coreutils --lang rust --destination uml.md"

echo "=== Benchmark: BuildCLI ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/BuildCLI --lang java --destination uml.md"

echo "=== Benchmark: faker-cxx ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/faker-cxx --lang cpp --destination uml.md"

echo "=== Benchmark: AuthPass ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/authpass --lang dart --destination uml.md"

echo "=== Benchmark: JupyterLab ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/jupyterlab --lang typescript --destination uml.md"

echo "=== Benchmark: bitwarden-server ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/bitwarden-server --lang csharp --destination uml.md"

echo "=== Benchmark: Platypus ==="
hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $BENCHMARK_DIR/Platypus --lang objective-c --destination uml.md"

echo ""

echo "Cleaning up benchmarks..."
rm -rf "$BENCHMARK_DIR"
rm uml.md

echo "Done!"
