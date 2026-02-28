use std::fs;
use tree_sitter::Parser as TreeSitterParser;
mod diagram;
mod lang_config;
mod mermaid;

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

    match args.lang.as_str() {
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
        _ => {
            println!("Error {} is not a supported language", args.lang);
            std::process::exit(404);
        }
    }

    // Parse source code

    let source = std::fs::read(args.source_code).expect("Failed to read source code file");

    let tree = parser.parse(&source, None).unwrap();

    // Get the root node and build diagram
    let root_node = tree.root_node();
    let mut program_diagram = Diagram::new(&args.lang);
    program_diagram.build(root_node, &source);

    // pass to the exporter and write
    let _ = fs::write(args.destination, mermaid::generate(&program_diagram));
}
