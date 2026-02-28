use auto_uml::diagram::Diagram;
use auto_uml::mermaid::generate;
use tree_sitter::Parser;

/// Configuration for a specific language's integration tests.
struct LangTestConfig {
    name: &'static str,
    path_prefix: &'static str,
    /// Mapping of (Test Category) -> (File Name)
    files: [(&'static str, &'static str); 4],
}

const RUST_TESTS: LangTestConfig = LangTestConfig {
    name: "rust",
    path_prefix: "test_source_code_examples/rust",
    files: [
        ("simple_struct", "simple_struct.rs"),
        ("impl_block", "impl_blocks.rs"),
        ("complex_types", "complex_types.rs"),
        ("generics", "generics.rs"),
    ],
};

const JAVA_TESTS: LangTestConfig = LangTestConfig {
    name: "java",
    path_prefix: "test_source_code_examples/java",
    files: [
        ("simple_struct", "User.java"),
        ("impl_block", "Calculator.java"),
        ("complex_types", "ComplexData.java"),
        ("generics", "Box.java"),
    ],
};

const JAVASCRIPT_TESTS: LangTestConfig = LangTestConfig {
    name: "javascript",
    path_prefix: "test_source_code_examples/javascript",
    files: [
        ("simple_struct", "User.js"),
        ("impl_block", "Calculator.js"),
        ("complex_types", "ComplexData.js"),
        ("generics", "Box.js"),
    ],
};

const CSHARP_TESTS: LangTestConfig = LangTestConfig {
    name: "csharp",
    path_prefix: "test_source_code_examples/csharp",
    files: [
        ("simple_struct", "User.cs"),
        ("impl_block", "Calculator.cs"),
        ("complex_types", "ComplexData.cs"),
        ("generics", "Box.cs"),
    ],
};

const CPP_TESTS: LangTestConfig = LangTestConfig {
    name: "cpp",
    path_prefix: "test_source_code_examples/cpp",
    files: [
        ("simple_struct", "User.cpp"),
        ("impl_block", "Calculator.cpp"),
        ("complex_types", "ComplexData.cpp"),
        ("generics", "Box.cpp"),
    ],
};

const TYPESCRIPT_TESTS: LangTestConfig = LangTestConfig {
    name: "typescript",
    path_prefix: "test_source_code_examples/typescript",
    files: [
        ("simple_struct", "User.ts"),
        ("impl_block", "Calculator.ts"),
        ("complex_types", "ComplexData.ts"),
        ("generics", "Box.ts"),
    ],
};

const ALL_LANGS: &[LangTestConfig] = &[
    RUST_TESTS,
    JAVA_TESTS,
    JAVASCRIPT_TESTS,
    CSHARP_TESTS,
    CPP_TESTS,
    TYPESCRIPT_TESTS,
];

fn setup_parser(lang: &str) -> Parser {
    let mut parser = Parser::new();
    match lang {
        "rust" => {
            parser
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .expect("Error loading Rust grammar");
        }
        "java" => {
            parser
                .set_language(&tree_sitter_java::LANGUAGE.into())
                .expect("Error loading Java grammar");
        }
        "javascript" => {
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .expect("Error loading JavaScript grammar");
        }
        "csharp" => {
            parser
                .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                .expect("Error loading C# grammar");
        }
        "cpp" => {
            parser
                .set_language(&tree_sitter_cpp::LANGUAGE.into())
                .expect("Error loading C++ grammar");
        }
        "typescript" => {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .expect("Error loading TypeScript grammar");
        }
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}

fn get_file_for_category(config: &LangTestConfig, category: &str) -> String {
    let file = config
        .files
        .iter()
        .find(|(cat, _)| *cat == category)
        .map(|(_, file)| *file)
        .expect(&format!(
            "No file defined for category {} in {}",
            category, config.name
        ));
    format!("{}/{}", config.path_prefix, file)
}

fn run_test(
    config: &LangTestConfig,
    category: &str,
    validator: impl FnOnce(&Diagram, &LangTestConfig),
) {
    let mut parser = setup_parser(config.name);
    let path = get_file_for_category(config, category);
    let source = std::fs::read(&path).expect(&format!("failed to read test file: {}", path));
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new(config.name);
    diagram.build(tree.root_node(), &source);

    validator(&diagram, config);
}

#[test]
fn test_all_simple_structs() {
    for config in ALL_LANGS {
        run_test(config, "simple_struct", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "User")
                .expect(&format!("Class 'User' not found for {}", cfg.name));
            assert_eq!(
                class.variables.len(),
                3,
                "Variable count mismatch for {}",
                cfg.name
            );
        });
    }
}

#[test]
fn test_all_impl_blocks() {
    for config in ALL_LANGS {
        run_test(config, "impl_block", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "Calculator")
                .expect(&format!("Class 'Calculator' not found for {}", cfg.name));
            assert!(
                class.functions.len() >= 2,
                "Function count low for {}",
                cfg.name
            );
            assert!(class.functions.iter().any(|f| f.name == "add"));
        });
    }
}

#[test]
fn test_all_complex_types() {
    for config in ALL_LANGS {
        run_test(config, "complex_types", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "ComplexData")
                .expect(&format!("Class 'ComplexData' not found for {}", cfg.name));
            assert!(class.variables.iter().any(|v| v.name == "raw_bytes"));
            assert!(class.functions.iter().any(|f| f.name == "process"));
        });
    }
}

#[test]
fn test_all_generics() {
    for config in ALL_LANGS {
        run_test(config, "generics", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name.contains("Box"))
                .expect(&format!("Generic class 'Box' not found for {}", cfg.name));
            assert!(class.variables.iter().any(|v| v.name == "inner"));
        });
    }
}

#[test]
fn test_mermaid_smoke() {
    // Just verify it doesn't crash and produces basic output for the first language
    run_test(&RUST_TESTS, "simple_struct", |diagram, _| {
        let output = generate(diagram);
        assert!(output.contains("classDiagram"));
        assert!(output.contains("User"));
    });
}
