use tree_sitter::Parser;

fn main() {
    // 1. Create the parser and set language
    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");

    // Parse souce code
    let source_code = "fn main() { let x = 1; }";
    let tree = parser.parse(source_code, None).unwrap();

    // 4. Get the root node
    let root_node = tree.root_node();

    println!("{}", root_node);
}
