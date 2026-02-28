use auto_uml::diagram::Diagram;
use auto_uml::mermaid::generate;
use tree_sitter::Parser;

fn setup_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");
    parser
}

#[test]
fn test_diagram_build_simple_struct() {
    let mut parser = setup_parser();
    let source = std::fs::read("test_source_code_examples/rust/simple_struct.rs")
        .expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    assert_eq!(diagram.classes.len(), 1);
    let class = &diagram.classes[0];
    assert_eq!(class.name, "User");
    assert_eq!(class.variables.len(), 3);
    assert_eq!(class.variables[0].name, "id");
    assert_eq!(class.variables[0].var_type, "u64");
    assert_eq!(class.variables[1].name, "username");
    assert_eq!(class.variables[1].var_type, "String");
    assert_eq!(class.variables[2].name, "email");
    assert_eq!(class.variables[2].var_type, "String");
}

#[test]
fn test_diagram_build_impl_blocks() {
    let mut parser = setup_parser();
    let source = std::fs::read("test_source_code_examples/rust/impl_blocks.rs")
        .expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    assert_eq!(diagram.classes.len(), 1);
    let class = &diagram.classes[0];
    assert_eq!(class.name, "Calculator");
    assert_eq!(class.functions.len(), 2);

    let add_func = &class.functions[0];
    assert_eq!(add_func.name, "add");
    assert_eq!(add_func.return_type, "i32");
    assert_eq!(add_func.arguments.len(), 2);
    assert_eq!(add_func.arguments[0].name, "a");
    assert_eq!(add_func.arguments[0].var_type, "i32");

    let clear_func = &class.functions[1];
    assert_eq!(clear_func.name, "clear");
}

#[test]
fn test_diagram_build_complex_types() {
    let mut parser = setup_parser();
    let source = std::fs::read("test_source_code_examples/rust/complex_types.rs")
        .expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    assert_eq!(diagram.classes.len(), 1);
    let class = &diagram.classes[0];
    assert_eq!(class.name, "ComplexData");
    assert_eq!(class.variables.len(), 1);
    assert_eq!(class.variables[0].name, "raw_bytes");
    assert!(class.variables[0].var_type.contains("Vec"));

    assert_eq!(class.functions.len(), 1);
    let func = &class.functions[0];
    assert_eq!(func.name, "process");
    assert_eq!(func.return_type, "bool");
    assert_eq!(func.arguments.len(), 1);
    assert_eq!(func.arguments[0].name, "mode");
    assert_eq!(func.arguments[0].var_type, "String");
}

#[test]
fn test_diagram_build_generics() {
    let mut parser = setup_parser();
    let source = std::fs::read("test_source_code_examples/rust/generics.rs")
        .expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    assert!(!diagram.classes.is_empty());
    let class = &diagram.classes[0];
    assert!(class.name.contains("Box"));

    assert_eq!(class.variables.len(), 1);
    assert_eq!(class.variables[0].name, "inner");
}

#[test]
fn test_mermaid_full_cycle() {
    let mut parser = setup_parser();
    let source = std::fs::read("test_source_code_examples/rust/simple_struct.rs")
        .expect("failed to read test file");
    let tree = parser.parse(&source, None).unwrap();
    let mut diagram = Diagram::new();
    diagram.build(tree.root_node(), &source);

    let output = generate(&diagram);

    assert!(output.contains("classDiagram"));
    assert!(output.contains("class User {"));
    assert!(output.contains("+id: u64"));
}
