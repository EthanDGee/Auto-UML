#!/bin/bash

set -e

RUNS=100
WARMUP=3

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_DIR="$SCRIPT_DIR/benchmarks"
TOOL_PATH="../target/release/auto-uml"

header() {
  echo ""
  echo "========================================"
  echo "$1"
  echo "========================================"
}

info() {
  echo ">>> $1"
}

build_latest() {
  header "Building Latest Release"
  cargo build --release
}

spinner_pid=0

spinner_start() {
  printf "%s " "$1"
  local spin='|/-\'
  local i=0
  while kill -0 "$spinner_pid" 2>/dev/null; do
    printf "\b%s" "${spin:i++%${#spin}:1}"
    sleep 0.1
  done
  printf "\bDone\n"
}

clone_repo() {
  local dir="$1"
  local url="$2"
  if [ ! -d "$dir" ]; then
    printf ">>> Cloning %s... " "$dir"
    git clone --depth 1 "$url" "$dir" &>/dev/null &
    spinner_pid=$!
    spinner_start
  else
    info "Skipping $dir (already exists)"
  fi
}

pull_codebases() {
  mkdir -p "$BENCHMARK_DIR"
  cd "$BENCHMARK_DIR"

  header "Pulling Codebases"

  clone_repo "coreutils" "https://github.com/uutils/coreutils.git"
  clone_repo "Chart.js" "https://github.com/chartjs/Chart.js"
  clone_repo "BuildCLI" "https://github.com/BuildCLI/BuildCLI"
  clone_repo "faker-cxx" "https://github.com/cieslarmichal/faker-cxx"
  clone_repo "ReactiveUI" "https://github.com/reactiveui/ReactiveUI"
  clone_repo "authpass" "https://github.com/authpass/authpass.git"
  clone_repo "jupyterlab" "https://github.com/jupyterlab/jupyterlab"
  clone_repo "bitwarden-server" "https://github.com/bitwarden/server"
  clone_repo "Platypus" "https://github.com/sveinbjornt/Platypus"
  clone_repo "ktlint" "https://github.com/ktlint/ktlint"

  cd "$SCRIPT_DIR"
}

# Maps an auto-uml language name to the file extensions scc should count.
lang_extensions() {
  case "$1" in
    rust) echo "rs" ;;
    javascript) echo "js,jsx,mjs,cjs" ;;
    typescript) echo "ts,tsx" ;;
    java) echo "java" ;;
    cpp) echo "cpp,cc,cxx,c++,hpp,hh,hxx,h" ;;
    csharp) echo "cs" ;;
    objective-c) echo "m,mm,h" ;;
    dart) echo "dart" ;;
    kotlin) echo "kt,kts" ;;
    ruby) echo "rb" ;;
    *) echo "" ;;
  esac
}

# Prints file/line counts for the language being benchmarked.
code_stats() {
  local source_code="$1"
  local lang="$2"

  if ! command -v scc &>/dev/null; then
    info "scc not installed, skipping code stats"
    return
  fi

  local exts
  exts="$(lang_extensions "$lang")"
  if [ -z "$exts" ]; then
    info "No extension mapping for $lang, skipping code stats"
    return
  fi

  local stats
  stats="$(scc --include-ext "$exts" --format json2 "$source_code" 2>/dev/null |
    jq -r '[.languageSummary[]] as $l
           | [([$l[].Count] | add // 0),
              ([$l[].Lines] | add // 0),
              ([$l[].Code]  | add // 0),
              ([$l[].Comment] | add // 0),
              ([$l[].Blank] | add // 0)]
           | @tsv')"

  if [ -z "$stats" ]; then
    info "Could not gather code stats for $source_code"
    return
  fi

  local files lines code comment blank
  IFS=$'\t' read -r files lines code comment blank <<<"$stats"

  printf ">>> %'d files | %'d lines | %'d code | %'d comments | %'d blank\n" \
    "$files" "$lines" "$code" "$comment" "$blank"
}

run_benchmark() {
  local name="$1"
  local source_code="$2"
  local lang="$3"
  header "Benchmark: $name ($lang)"
  code_stats "$source_code" "$lang"
  hyperfine --runs $RUNS --warmup $WARMUP "$TOOL_PATH --source-code $source_code --lang $lang --destination uml.md"
}

run_benchmarks() {
  header "Running Benchmarks"

  run_benchmark "This Codebase" "../src/" "rust"
  run_benchmark "Chart.js" "$BENCHMARK_DIR/Chart.js" "javascript"
  run_benchmark "CoreUtils" "$BENCHMARK_DIR/coreutils" "rust"
  run_benchmark "BuildCLI" "$BENCHMARK_DIR/BuildCLI" "java"
  run_benchmark "faker-cxx" "$BENCHMARK_DIR/faker-cxx" "cpp"
  run_benchmark "AuthPass" "$BENCHMARK_DIR/authpass" "dart"
  run_benchmark "JupyterLab" "$BENCHMARK_DIR/jupyterlab" "typescript"
  run_benchmark "bitwarden-server" "$BENCHMARK_DIR/bitwarden-server" "csharp"
  run_benchmark "Platypus" "$BENCHMARK_DIR/Platypus" "objective-c"
  run_benchmark "ktlint" "$BENCHMARK_DIR/ktlint" "kotlin"
}

cleanup() {
  header "Cleaning Up"
  info "Removing benchmark directory..."
  rm -rf "$BENCHMARK_DIR"
  info "Removing output file..."
  rm -f uml.md
}

build_latest
pull_codebases
run_benchmarks
cleanup

header "All Done!"
