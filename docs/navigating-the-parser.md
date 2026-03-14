# Navigating the Parser

The core of `auto-UML` is the `Diagram` builder, which transforms raw source code into a structured representation of classes, functions, and variables.

## What is Tree-Sitter?

[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) is a modern parser generator tool. Unlike traditional parsers, it builds a **Concrete Syntax Tree (CST)** that preserves almost everything from the source code, including comments and whitespace. In this project, we use it to perform "lightweight" static analysis across many different programming languages without requiring a full compiler toolchain.

## Why is it so efficient?

* **Incremental Parsing**: While primarily designed for editors to re-parse files on every keystroke, Tree-sitter provides lightning-fast "one-shot" parsing for our CLI.
* **Language Agnostic**: By using a uniform API for different grammars, we can use the exact same navigation logic for Rust, Java, or C++ just by swapping the configuration.
* **Robustness**: It handles syntax errors gracefully, allowing `auto-UML` to generate diagrams even if the source code is currently in a "broken" or partial state.

## What is a Node?

A **Node** is the fundamental building block of the syntax tree. It represents a specific range of source code and has a `kind` (a string label like `class_definition`, `identifier`, or `parameter`).

Nodes are organized in a parent-child hierarchy. For example, a `function_definition` node will have children representing its name, its parameter list, and its body.

## Handling Node Types

The `navigate_node` method recursively traverses the tree. It consults the `LangConfig` (defined in YAML) to determine how to handle each node based on its `kind`:

### Imports & Namespaces

* **Imports**: Patterns like `use_declaration` or `import_statement` are collected to track external dependencies.
* **Namespaces**: Patterns like `mod_item` or `namespace_definition` help build a "qualified" name for classes (e.g., `auth_User`) to prevent name collisions in large projects.

### Classes & Structs

When a class pattern is matched, the parser creates a new `Class` entry. As the parser continues to recurse into the children of this node, it maintains a "class context." Any functions or variables found within that subtree are automatically attributed to that class.

### Functions & Methods

For every function node, the parser:

1. Extracts the **Identifier** (the function name).
2. Extracts the **Return Type** by searching for type-specific nodes.
3. Dives into the **Parameters** container to extract the name and type of every argument.

### Variables & Fields

Variable nodes are processed for their name and type. If the parser is currently within a class context, these are added as member variables (fields) to the UML class.

## Extraction Helpers

* **`extract_identifier`**: A specialized helper that searches a node's children for names. It is designed to "look through" wrapper nodes (like declarators) that some languages use to nest the actual identifier.
* **`extract_type`**: Identifies type nodes and includes logic for parsing **Generics** (e.g., converting `Vec<String>` into a base type and inner types), which are rendered as `Vec~String~` in Mermaid.
