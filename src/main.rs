use std::fs;
use tree_sitter::Parser as TreeSitterParser;
mod diagram;
mod mermaid;

enum Language {
    Rust,
    Cpp,
    Java,
}

use clap::Parser;

use crate::diagram::Diagram;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Programming language of the source file
    #[arg(short, long)]
    lang: String,
    /// Path to the source file
    #[arg(short, long)]
    source_code: String,

    /// Destination file for the exporter
    #[arg(short, long)]
    destination: String,
}

fn main() {
    let args = Args::parse();
    //  Create the parser and set language
    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");

    // Parse source code

    let source = std::fs::read(args.source_code).expect("Failed to read source code file");

    let tree = parser.parse(&source, None).unwrap();

    // Get the root node and build diagram
    let root_node = tree.root_node();
    let mut program_diagram = Diagram::new();
    program_diagram.build(root_node, &source);

    // pass to the exporter and write
    let _ = fs::write(args.destination, mermaid::generate(&program_diagram));
}

