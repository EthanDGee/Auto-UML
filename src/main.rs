use std::fs;
use tree_sitter::Parser as TreeSitterParser;
mod diagram;

enum Language {
    Rust,
    Cpp,
    Java,
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Programming language of the source file
    #[arg(short, long)]
    lang: String,
    /// Path to the source file
    #[arg(short, long)]
    file_path: String,
}

fn main() {
    let args = Args::parse();
    // 1. Create the parser and set language
    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");

    // Parse source code
    let source_code = fs::read_to_string(&args.file_path).expect("Failed to read file");
    let tree = parser.parse(&source_code, None).unwrap();

    // 4. Get the root node
    let root_node = tree.root_node();

    println!("{}", root_node);

    // pass to the
}
