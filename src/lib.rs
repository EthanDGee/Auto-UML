//! # Auto-UML
//!
//! A lightning fast automatic UML diagram generator that uses tree-sitter to parse source code and generate Mermaid class diagrams.
//!
//! This tool analyzes source code and generates class diagrams in Mermaid format.
//! It supports multiple programming languages and can work with local files/directories
//! or remote git repositories.
//!
//! ## Features
//!
//! - **Multi-language support**: rust, java, javascript, typescript, cpp, csharp, objective-c, and dart.
//! - **Single file analysis**: Generate UML diagrams from individual source files.
//! - **Repository-scale analysis**: Process entire directories and merge diagrams across multiple files.
//! - **Remote Analysis**: Clone and analyze remote git repositories directly.
//! - **Type resolution**: Automatically resolves types across files in multi-file projects.
//! - **Automatic language detection**: Detects the programming language from file extensions.
//!
//! ## Installation
//!
//! ```bash
//! cargo install auto-uml
//! ```
//!
//! ## Usage
//!
//! ### Analyze current directory (auto-detect language)
//! ```bash
//! auto-uml
//! ```
//!
//! ### Single file mode (language auto-detected)
//! ```bash
//! auto-uml --source-code path/to/file.rs
//! ```
//!
//! ### Remote git repository
//! ```bash
//! auto-uml --git https://github.com/user/repo.git
//! ```
//!
//! ### Specify destination (defaults to UML.md)
//! ```bash
//! auto-uml --destination my_diagram.md
//! ```
//!
//! ## Supported Languages
//!
//! | Language    | Extensions        |
//! |-------------|------------------|
//! | rust        | `.rs`            |
//! | java        | `.java`          |
//! | javascript  | `.js`            |
//! | typescript  | `.ts`, `.tsx`    |
//! | cpp         | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.h` |
//! | csharp      | `.cs`            |
//! | objective-c | `.m`, `.h`       |
//! | dart        | `.dart`          |
//!
//! ## Output
//! By default, outputs a Mermaid code block in markdown. Use `--no-mermaid` for raw Mermaid syntax.
//!
//! ## How It Works
//!
//! 1. **Parsing**: Uses tree-sitter to build abstract syntax trees (AST) from source code.
//! 2. **Extraction**: Traverses the AST to extract classes, functions, methods, and variables using language specific config.
//! 3. **Stitching**: For multi-file projects, merges diagrams and resolves types across files to create one diagram.
//! 4. **Generation**: Outputs into a Mermaid.js class diagram.
//!
//! ## Example
//!
//! ### Input (Rust):
//!
//! ```rust
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! impl User {
//!     fn new(name: String, email: String) -> User {
//!         User { name, email }
//!     }
//!
//!     fn greet(&self) -> String {
//!         format!("Hello, {}!", self.name)
//!     }
//! }
//! ```
//!
//! ### Output (Mermaid):
//!
//! ```mermaid
//! classDiagram
//!     class User {
//!         +name: String
//!         +email: String
//!         +new(name: String, email: String) User
//!         +greet() String
//!     }
//! ```

pub mod diagram;
pub mod lang_config;
pub mod mermaid;
pub mod stitcher;
