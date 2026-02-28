use std::fs;
use tree_sitter::Parser as TreeSitterParser;
mod diagram;
mod lang_config;
mod mermaid;
mod stitcher;
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
    
    // Parse source code
    let input_path = std::path::PathBuf::from(&args.source_code);
    let final_diagram = if input_path.is_dir() {
        let mut stitcher = stitcher::Stitcher::new(input_path, args.lang.clone());
        let mut directory = stitcher.build();
        directory.merge_all();
        directory.resolve_types(&stitcher.type_map);
        directory.merged_diagram
    } else {
        // Single file mode
        let mut parser = TreeSitterParser::new();
        match args.lang.to_lowercase().as_str() {
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
            "js" | "javascript" => {
                parser
                    .set_language(&tree_sitter_javascript::LANGUAGE.into())
                    .expect("Error loading javascript grammar");
            }
            "ts" | "typescript" => {
                parser
                    .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                    .expect("Error loading typescript grammar");
            }
            "c++" | "cpp" => {
                parser
                    .set_language(&tree_sitter_cpp::LANGUAGE.into())
                    .expect("Error loading c++ grammar");
            }
            "c#" | "cs" | "c-sharp" => {
                parser
                    .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                    .expect("Error loading c# grammar");
            }
            _ => {
                println!("Error {} is not a supported language", args.lang);
                std::process::exit(404);
            }
        }

        let source = std::fs::read(&args.source_code).expect("Failed to read source code file");
        let tree = parser.parse(&source, None).unwrap();
        let root_node = tree.root_node();
        let mut program_diagram = Diagram::new(&args.lang);
        program_diagram.build(root_node, &source);
        program_diagram
    };

    // pass to the exporter and write
    let _ = fs::write(args.destination, mermaid::generate(&final_diagram));
}
