# Add a language

Adding a language is as simple as creating a few YAMLs (2 to be exact) and some example code for integration testing to ensure accuracy. There's rarely any need to modify the code base. The initial languages were all added after creating the first config and necessitated no changes to the parser.

There are 5 simple steps

1. [Add the tree-sitter crate](#adding-the-tree-sitter-crate)
1. [Create language-specific examples codebase for testing](#create-the-language-specific-testing)
1. [Define the language config YAML](#defining-the-language-config)
1. [Add the language to the CLI and selector](#add-the-language-to-the-cli-and-selector)
1. [Prepare your merge request](#prepare-your-merge-request)

## Adding the tree-sitter crate

Due to the fact that every language is slightly different, tree-sitter parsers need to be custom built for every language.

Fortunately, this is already handled for most languages by the Rust community. So all that is required to add these language parsers is to track down the language-specific parser. Typically these are called `tree-sitter-<lang>` (e.g., `tree-sitter-python`).

To add these to the project all that is required is to run the following command:

```fish
cargo add tree-sitter-<lang>
```

This will automatically add the latest version of the parser into the project's configuration so that it can be easily used.

## Create the language-specific testing

In order to make sure that the config is accurate and can be deployed. We first must create accurate integration tests to see its real-world accuracy. It might be tempting to do other steps first but it is **HIGHLY RECOMMENDED** to create the test suite before any other steps as it is critical that we define appropriate output first. Creating tests second leads to a tendency to settle for mediocre results.

We have a standardized set of cases to consider for our configs. We cover both specific smaller tests like ensuring matching typing as well as full end-to-end matches with UMLs. You will need to create 4 matching source files that cover the 4 following aspects as well as their associated UML diagrams.

- **User**: Focuses on **basic attributes and data-only structures**. This is a simple class with several fields of different basic types (e.g., `long`, `string`) to verify that fields are correctly parsed and categorized within the diagram.
- **Calculator**: Focuses on **methods and signatures**. This example should contain a class with several methods that include multiple parameters and return types. It ensures that the parser correctly extracts function names, their arguments, and their return values.
- **Complex Data**: Focuses on **standard library types and member visibility**. This example uses more complex, nested types such as `std::vector<unsigned char>` or `std::string` and tests whether the parser correctly handles private and public access specifiers.
- **Box**: Focuses on **generics and templates**. This example should demonstrate how the language handles generic types (e.g., `template <typename T>` in C++ or `struct Box<T>` in Rust) and how they should be correctly represented in the resulting UML (e.g., `Box~T~`).

### Where to put your tests

All tests and examples should be located in a language-specific directory under the `languages/` folder. For a new language, you should create the following structure:

```text
languages/
└── <language-name>/
    ├── config.yaml
    ├── test.yaml
    └── examples/
        ├── Box.<extension>
        ├── Box.uml
        ├── Calculator.<extension>
        ├── Calculator.uml
        ├── ComplexData.<extension>
        ├── ComplexData.uml
        ├── User.<extension>
        └── User.uml
```

### Setting up the test.yaml

The `test.yaml` file is crucial for the integration testing suite. It's what is used to map your example files to their associated tests in `tests/integration_tests.rs`. Using the C++ example (`languages/cpp/test.yaml`):

```yaml
name: cpp
path_prefix: languages/cpp/examples
files:
  simple_struct: User.cpp
  impl_block: Calculator.cpp
  complex_types: ComplexData.cpp
  generics: Box.cpp
```

> For each source file, the test runner will look for a matching `.uml` file (e.g., `User.uml`) in the same directory to validate the generated output against your expected Mermaid diagram.

### Add to Integration Tests in `tests/integration_tests.rs`

To ensure your language is included in the test suite, update the `load_test_configs` and `setup_parser` functions in `tests/integration_tests.rs`.

```rust
fn load_test_configs() -> Vec<LangTestConfig> {
    let mut configs = Vec::new();
    let langs = [
        // ... existing languages ...
        "your-lang",
    ];
    // ...
}

fn setup_parser(lang: &str) -> Parser {
    let mut parser = Parser::new();
    match lang {
        // ... existing languages ...
        "your-lang" => {
            parser
                .set_language(&tree_sitter_your_lang::LANGUAGE.into())
                .expect("Error loading your-lang grammar");
        }
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}
```

## Defining the language config

Additionally, since every language has its own eccentricities and language-specific parser. The tree-sitter nodes have different naming conventions. This is overall a good thing as it makes it much easier for language-specific functionality. As a result a language-specific config needs to be created.

This is done in the form of a YAML file, which auto-UML uses to navigate the tree according to that language's conventions.

> These configs are compiled into the binary to ensure portability. `build.rs` globs every `languages/*/config.yaml` into `$OUT_DIR/staged_languages`, and `src/lang_config.rs` embeds that directory with `include_dir!`.

The following is the Rust config. Each of these labels is the node names for that type

```yaml
file_extensions:
  - rs
class_patterns:
  - struct_item
  - impl_item
  - class_declaration
  - class_specifier
function_patterns:
  - function_item
  - method_declaration
  - function_definition
variable_patterns:
  - field_declaration
  - variable_declaration
identifier_patterns:
  - identifier
  - field_identifier
  - type_identifier
type_patterns:
  - type
  - primitive_type
parameter_container_patterns:
  - parameters
  - formal_parameters
parameter_patterns:
  - parameter
  - formal_parameter
wrapper_patterns:
  - generic_type
  - variable_declarator
  - field_declaration
  - function_item
  - method_declaration
  - class_declaration
  - struct_item
skip_patterns: # some nodes do not need to be handled and can be skipped
  - modifiers
  - visibility_modifier
  - storage_class
  - attribute_item
  - type_parameters
import_patterns:
  - use_declaration
namespace_patterns:
  - mod_item
visibility_modifier_patterns:
  - visibility_modifier
private_by_default: true
public_modifier_patterns:
  - pub
class_type_parameter_patterns:
  - type_parameters
type_path_separator: "::"
self_parameter_patterns:
  - self_parameter
```

> **Every field is optional.** All keys not added to configs will not be used as part of the parsing process. This makes configuration far more modular and keeps languages such as javascript which don't need most features fast, and efficient while still allowing the handling of more complex language like rust.

> YAML keys map one-to-one onto the snake_case fields of `LangConfig` in `src/lang_config.rs`. There are no aliases or renames, so the struct is the authoritative schema.

### Config field reference

Every field below is optional. Unless noted, each takes a list of tree-sitter node
kinds. Start with the structural fields; reach for the quirk toggles at the bottom
only when a grammar forces you to.

#### Structure and scoping

| Field | Purpose |
| ----- | ------- |
| `class_patterns` | Node kinds that open a class, struct, or impl block. Everything found inside becomes a member of that class. |
| `function_patterns` | Node kinds that declare a function or method. |
| `variable_patterns` | Node kinds that declare a field or variable. |
| `parameter_container_patterns` | Node kinds holding a function's parameter list. |
| `parameter_patterns` | Node kinds for an individual parameter inside that list. |
| `wrapper_patterns` | Nodes that only nest the thing you actually want. The walker recurses through them instead of treating them as a declaration. |
| `skip_patterns` | Nodes to ignore outright, so they never reach the class/function/variable handlers. |
| `import_patterns` | Node kinds for import statements, collected to resolve types across files. |
| `namespace_patterns` | Node kinds that introduce a namespace/module. Used to build qualified class names such as `auth_User`. |
| `constructor_patterns` | Node kinds that are constructors. Their return type is suppressed, so no stray trailing `void`. |
| `self_parameter_patterns` | Node kinds for an implicit receiver (Rust's `self`), excluded from the rendered parameter list. |

#### Identifiers and types

| Field | Purpose |
| ----- | ------- |
| `file_extensions` | Extensions used by the language.   |
| `identifier_patterns` | Node kinds that carry a declared name. |
| `type_patterns` | Node kinds that carry a type. The literal entry `type` is special-cased: it also matches any node kind *containing* `type`, which Rust's grammar relies on. |
| `type_field_names` | Grammar field names marking a child as a *type reference* rather than a declared name. Needed when a grammar reuses one node kind (`identifier`) for both, as C# does. |
| `type_argument_patterns` | Node kinds holding generic arguments as a *sibling* of the type name rather than inside it — Dart parses `List<int>` as `type_identifier` plus `type_arguments`. |
| `type_strip_suffixes` | Trailing text attached to a type node that is not part of the name. |
| `type_annotation_strip_prefix` | Prefix strings that should be stripped from type annotations. |
| `type_path_separator` | The language's path separator, e.g. `"::"` for Rust or `"."` for Java, used when qualifying names. |
| `class_type_parameter_patterns` | Node kinds holding a class's own generic parameters, rendered as `Box~T~`. |
| `builtin_type_names` | Standard-library type names such as `String` or `Object` that shouldn't be treated as user defined classes. |

#### Visibility

| Field | Purpose |
| ----- | ------- |
| `visibility_modifier_patterns` | Node kinds that carry a visibility keyword. |
| `public_modifier_patterns` | The keyword text that means public, e.g. `pub`. |
| `private_modifier_patterns` | The keyword text that means private. |
| `private_by_default` | Boolean. `true` when members are private unless marked otherwise. |

#### Language quirk toggles

All four are booleans, all default to off.

| Field | Purpose |
| ----- | ------- |
| `infer_param_types_from_fields` | The language has field-promoting parameters, such as Dart's `Box(this.inner)`. An untyped parameter adopts the type of the same-named field on the enclosing class. |
| `suppress_self_relations` | Drop association and dependency edges where source and destination are the same class. |
| `infer_unresolved_type_relations` | Draw an association to a field's type even when that class is not in the parsed diagram, e.g. an imported type. Skipped for generic containers, lowercase primitives, and anything in `builtin_type_names`. |
| `loosely_typed` | The language has no real type annotations (JavaScript). Suppresses the `void` placeholder that would otherwise render as `+id: void`. |

## Add the language to the CLI and selector

Once you have your config and tests ready, you need to register the language within the `auto-UML` codebase.

### 1. Register the tree-sitter Parser in `src/main.rs`

Update the `get_parser` function in `src/main.rs` to include your new language and its corresponding tree-sitter crate.

```rust
fn get_parser(lang: &str) -> TreeSitterParser {
    let mut parser = TreeSitterParser::new();
    match lang.to_lowercase().as_str() {
        // ... existing languages ...
        "your-lang" => {
            parser
                .set_language(&tree_sitter_your_lang::LANGUAGE.into())
                .expect("Error loading your-lang grammar");
        }
        _ => {
            println!("Error {} is not a supported language", lang);
            std::process::exit(404);
        }
    }
    parser
}
```

### 2. Update the `LangConfig` in `src/lang_config.rs`

Add your language to the `load` function in `src/lang_config.rs`.

```rust
impl LangConfig {
    pub fn load(language: &str) -> Self {
        let lang_dir = match language.to_lowercase().as_str() {
            // ... existing mappings ...
            "your-lang" => "your-lang",
            _ => language,
        };
        // ...
    }

}
```

### 3. Add to `list_languages` in `LangConfig`

This is used for the `--list-languages` help flag in the CLI tool.

```rust
impl LangConfig {
  pub fn list_languages() -> Vec<&'static str> {
        vec![
            // ... existing languages ...
            "your-lang",
        ]
    }
}
```

## Prepare your merge request

Once you've verified your changes:

1. **Run tests**: Execute `cargo test` to ensure all integration tests pass for your new language.
2. **Linting**: Ensure your code follows the project standards. Ideally, the only changes to the source code should be adding it to the CLI tool, parser loader, and testing. Anything else will be highly scrutinized. So if changes were made, make sure to explain them in your merge request.
3. **Submission**: Submit your merge request. Include both your code changes and the new `languages/<language-name>/` directory with its configuration and examples. The merge request name should be something along the lines of `feat: Added support for <language-name>`.
