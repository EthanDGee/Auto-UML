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

const ALL_LANGS: &[LangTestConfig] = &[RUST_TESTS, JAVA_TESTS];

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
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    validator(&diagram, config);
}

#[test]
fn debug_java_tree() {
    let mut parser = setup_parser("java");
    let path = "test_source_code_examples/java/User.java";
    let source = std::fs::read(&path).expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();

    fn print_node(node: tree_sitter::Node, source: &[u8], depth: usize) {
        let kind = node.kind();
        let text = if node.child_count() == 0 {
            format!(
                ": '{}'",
                String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()])
            )
        } else {
            "".to_string()
        };
        println!("{}{}{}", "  ".repeat(depth), kind, text);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            print_node(child, source, depth + 1);
        }
    }
    print_node(tree.root_node(), &source, 0);
}

#[test]
fn debug_java_box() {
    let mut parser = setup_parser("java");
    let path = "test_source_code_examples/java/Box.java";
    let source = std::fs::read(&path).expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();

    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    for class in &diagram.classes {
        println!("Class: {}", class.name);
        for var in &class.variables {
            println!("  Var: {} : {}", var.name, var.var_type);
        }
        for func in &class.functions {
            println!("  Func: {}() : {}", func.name, func.return_type);
        }
    }
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
