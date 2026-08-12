# Auto-UML

<!--toc:start-->
- [Auto-UML](#auto-uml)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Full Options](#full-options)
  - [Supported Languages](#supported-languages)
  - [Output](#output)
    - [Ignored files](#ignored-files)
  - [How It Works](#how-it-works)
  - [Example](#example)
  - [Benchmark](#benchmark)
    - [Test codebases](#test-codebases)
    - [Codebase statistics](#codebase-statistics)
    - [Results](#results)
<!--toc:end-->

A lightning fast automatic UML diagram generator that uses tree-sitter to parse source code and generate Mermaid or Graphviz class diagrams.

## Features

- **Multi-language support**: Rust, Java, JavaScript, TypeScript, C++, C#, Objective-C, Dart, and Kotlin.
- **Single file analysis**: Generate UML diagrams from individual source files.
- **Repository-scale analysis**: Process entire directories and merge diagrams across multiple files.
- **Remote Analysis**: Clone and analyze remote git repositories directly.
- **Type resolution**: Automatically resolves types across files in multi-file projects.
- **Automatic language detection**: Detects the programming language from file extensions.
- **Multiple output formats**: Mermaid class diagrams or Graphviz DOT, raw or wrapped in a markdown code block.
- **Ignore-aware**: Respects `.gitignore` and other ignore files, so build artifacts and vendored code stay out of the diagram.

## Installation

```fish
cargo install auto-uml
```

Or alternatively to build from source.

```fish
cargo install --git https://github.com/EthanDGee/Auto-UML.git
```

## Usage

```fish
# Analyze current directory (auto-detect language)
auto-uml

# Analyze the current directory with an explicit language
auto-uml --lang rust

# List the languages this build supports
auto-uml --list-languages

# Single file mode (language auto-detected)
auto-uml --source-code path/to/file.rs

# Directory mode (entire repository)
auto-uml --source-code path/to/project

# Remote git repository
auto-uml --git https://github.com/user/repo.git

# Specify destination (Defaults to UML.mmd)
auto-uml --destination my_diagram.mmd

# Wrap the diagram in a markdown code block (writes UML.md)
auto-uml --code-block

# Export Graphviz DOT instead of Mermaid (writes UML.dot)
auto-uml --format graphviz

# Include files that .gitignore and friends would otherwise exclude
auto-uml --no-ignore
```

### Full Options

| Flag | Description |
| ------ | ------------- |
| `--list-languages` | List available programming languages, then exit |
| `-l`, `--lang <LANG>` | Programming language (optional, auto-detected if omitted) |
| `-s`, `--source-code <PATH>` | Path to the local source file or directory [default: current directory]. Mutually exclusive with `--git` |
| `--git <URL>` | Remote git repository URL to clone and analyze |
| `-f`, `--format <FORMAT>` | Output format, `mermaid` or `graphviz` [default: mermaid] |
| `--code-block` | Wraps the output in a markdown code block. Off by default |
| `--no-ignore` | Include files excluded by `.gitignore` and other ignore files |
| `-d`, `--destination <FILE>` | Write destination file [default: `UML.mmd`, or `UML.dot` for graphviz, or `UML.md` with `--code-block`] |

## Supported Languages

| Language    | Extensions                          |
|-------------|-------------------------------------|
| cpp         | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.h` |
| rust        | `.rs`                               |
| java        | `.java`                             |
| javascript  | `.js`                               |
| typescript  | `.ts`, `.tsx`                       |
| csharp      | `.cs`                               |
| objective-c | `.m`, `.h`                          |
| dart        | `.dart`                             |
| kotlin      | `.kt`, `.kts`                       |

## Output

By default the tool writes a Mermaid class diagram as raw text to `UML.mmd`. You can render it with:

- [Mermaid Live Editor](https://mermaid.live/)
- VS Code with Mermaid extension
- GitHub

Pass `--code-block` to wrap the diagram in a markdown fence instead (written to `UML.md`),
which is what GitHub needs to render it inline.

Pass `--format graphviz` for Graphviz DOT (written to `UML.dot`), which you can render with
the `dot` command:

```fish
auto-uml --format graphviz
dot -Tsvg UML.dot -o UML.svg
```

### Ignored files

Files excluded by `.gitignore`, `.ignore`, and other standard ignore files are skipped in
directory mode. Pass `--no-ignore` to include them.

## How It Works

1. **Parsing**: Uses tree-sitter to build a concrete syntax tree (CST) from source code.
2. **Extraction**: Traverses the CST to extract classes, functions, methods, and variables using language specific config.
3. **Stitching**: For multi-file projects, merges diagrams and resolves types across files to create one diagram.
4. **Generation**: Renders the finished diagram as Mermaid or Graphviz DOT, depending on `--format`.

## Example

Input (Rust):

```rust
struct User {
    name: String,
    email: String,
}

impl User {
    fn new(name: String, email: String) -> User {
        User { name, email }
    }

    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}
```

Output (Mermaid):

```mermaid
classDiagram
    class User {
        +name: String
        +email: String
        +new(name: String, email: String) User
        +greet() String
    }
```

## Benchmark

Auto-UML is designed from the ground up to be extremely fast. Since it works with a per-file LR representation that is merged recursively it can also work on extremely large code bases. Most code bases can be done in milliseconds with the larger ones taking seconds. Making it a great addition to your CI/CD pipeline or for personal use.

> **Disclaimer**: All benchmarks were performed on the latest main branch as of August 12, 2026.

### Test code bases

All of the following tests were done using hyperfine with 100 runs.

- This Codebase - As is tradition for many analysis programs it is fitting that this program is used to analyze itself.

- [CoreUtils](https://github.com/uutils/coreutils.git) - A Rust rewrite of the GNU core utilities.

- [Chart.js](https://github.com/chartjs/Chart.js) - A popular JavaScript charting library. Tests done using the JavaScript parser.

- [BuildCLI](https://github.com/BuildCLI/BuildCLI) - A Java CLI framework.

- [faker-cxx](https://github.com/cieslarmichal/faker-cxx) - A C++ fake data generator library.

- [authpass](https://github.com/authpass/authpass.git) - A Flutter/Dart password manager.

- [jupyterlab](https://github.com/jupyterlab/jupyterlab) - A large TypeScript project.

- [bitwarden-server](https://github.com/bitwarden/server) - A large C# project.

- [Platypus](https://github.com/sveinbjornt/Platypus) - An Objective-C macOS application.

- [ktlint](https://github.com/pinterest/ktlint) - An anti-bikeshedding Kotlin linter with built-in formatter.

### Codebase Statistics

| Codebase                                                      | Files | Lines    | Code     | Comments | Blank    |
| ------------------------------------------------------------ | ----- | -------- | -------- | -------- | -------- |
| [This Codebase](https://github.com/anomalyco/auto-UML)      | 7     | 2,333    | 1,778    | 364      | 191      |
| [Chart.js](https://github.com/chartjs/Chart.js)             | 658   | 64,944   | 56,341   | 3,344    | 5,259    |
| [CoreUtils](https://github.com/uutils/coreutils.git)         | 638   | 255,332  | 199,034  | 28,316   | 27,982   |
| [BuildCLI](https://github.com/BuildCLI/BuildCLI)            | 250   | 16,143   | 12,599   | 648      | 2,896    |
| [faker-cxx](https://github.com/cieslarmichal/faker-cxx)     | 174   | 117,490  | 107,431  | 4,403    | 5,656    |
| [authpass](https://github.com/authpass/authpass.git)         | 162   | 62,697   | 47,188   | 3,288    | 12,221   |
| [jupyterlab](https://github.com/jupyterlab/jupyterlab)       | 993   | 310,228  | 213,609  | 64,926   | 31,693   |
| [bitwarden-server](https://github.com/bitwarden/server)     | 5,277 | 1,862,716 | 1,365,077 | 38,035  | 459,604  |
| [Platypus](https://github.com/sveinbjornt/Platypus)         | 135   | 19,815   | 10,192   | 6,534    | 3,089    |
| [ktlint](https://github.com/pinterest/ktlint)                 | 433   | 88,760   | 76,945   | 5,481    | 6,334    |

### Results

| Codebase                                                      | Language      | Mean runtime | Lines/sec | Standard Deviation | Min       | Max       |
| ------------------------------------------------------------ | ------------- | ------------ | --------- | ------------------ | --------- | --------- |
| [This Codebase](https://github.com/anomalyco/auto-UML)      | Rust          | 13.4 ms      | 174,104   | 1.4 ms             | 10.9 ms   | 17.7 ms   |
| [Chart.js](https://github.com/chartjs/Chart.js)             | JavaScript    | 212.3 ms     | 305,907   | 4.0 ms             | 202.2 ms  | 220.8 ms  |
| [CoreUtils](https://github.com/uutils/coreutils.git)         | Rust          | 1.031 s      | 247,655   | 0.122 s            | 0.931 s   | 1.469 s   |
| [BuildCLI](https://github.com/BuildCLI/BuildCLI)            | Java          | 73.7 ms      | 219,037   | 5.9 ms             | 66.0 ms   | 91.8 ms   |
| [faker-cxx](https://github.com/cieslarmichal/faker-cxx)     | C++           | 815.4 ms     | 144,089   | 31.4 ms            | 768.9 ms  | 960.7 ms  |
| [authpass](https://github.com/authpass/authpass.git)         | Dart          | 297.2 ms     | 210,959   | 11.9 ms            | 282.4 ms  | 369.3 ms  |
| [jupyterlab](https://github.com/jupyterlab/jupyterlab)       | TypeScript    | 1.170 s      | 265,152   | 0.066 s            | 1.114 s   | 1.497 s   |
| [bitwarden-server](https://github.com/bitwarden/server)     | C#            | 16.122 s     | 115,539   | 0.708 s            | 15.200 s  | 19.555 s  |
| [Platypus](https://github.com/sveinbjornt/Platypus)         | Objective-C   | 156.2 ms     | 126,857   | 2.8 ms             | 150.7 ms  | 164.5 ms  |
| [ktlint](https://github.com/pinterest/ktlint)                 | Kotlin        | 702.7 ms     | 126,313   | 12.9 ms            | 678.2 ms  | 757.1 ms  |
