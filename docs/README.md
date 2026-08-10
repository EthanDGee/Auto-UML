# Documentation Overview

Welcome to the `auto-UML` documentation. This project is a lightning-fast automatic UML diagram generator that uses Tree-sitter to parse source code and generate Mermaid or Graphviz class diagrams.

## Key Concepts

`auto-UML` works by traversing the Concrete Syntax Tree (CST) of your source code. It uses language-specific configurations to identify classes, functions, and variables, even across multiple files.

## Documentation Structure

* **[Contributing](../CONTRIBUTING.md)**: Local setup, the test commands, and the prek hooks your changes need to pass before they can be pushed.
* **[Adding a Language](./add-a-language.md)**: A step-by-step guide for developers who want to extend `auto-UML` support to a new programming language. It covers tree-sitter integration, configuration YAMLs, and integration testing.
* **[Navigating the Parser](./navigating-the-parser.md)**: A deep dive into how the core engine uses Tree-sitter to extract structured data from raw source code. It explains the hierarchy of nodes and the recursive navigation strategy.

## Getting Started

If you are a user looking for installation and usage instructions, please refer to the main **[README.md](../README.md)** at the root of the repository.

If you are a contributor, start with **[CONTRIBUTING.md](../CONTRIBUTING.md)** for setup and the required checks, then move on to the **[Add a Language](./add-a-language.md)** guide.
