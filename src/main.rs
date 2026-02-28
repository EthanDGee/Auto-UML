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
    /// Programming language (optional, auto-detected if omitted)
    #[arg(short, long)]
    lang: Option<String>,
    /// Path to the source file or directory
    #[arg(short, long)]
    source_code: String,

    /// Destination file for the exporter
    #[arg(short, long)]
    destination: String,
}

fn detect_language(path: &std::path::Path) -> Option<String> {
    if path.is_file() {
        let ext = path.extension()?.to_str()?;
        return match ext.to_lowercase().as_str() {
            "rs" => Some("rust".to_string()),
            "java" => Some("java".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "cpp" | "cc" | "cxx" | "hpp" | "h" => Some("cpp".to_string()),
            "cs" => Some("csharp".to_string()),
            _ => None,
        };
    }

    if path.is_dir() {
        // Look for the first supported file extension in the directory
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(l) = detect_language(&p) {
                        return Some(l);
                    }
                } else if let Some(l) = detect_language(&p) {
                    return Some(l);
                }
            }
        }
    }
    None
}

fn main() {
    let args = Args::parse();

    // Parse source code
    let input_path = std::path::PathBuf::from(&args.source_code);

    let lang = args.lang.clone().or_else(|| {
        detect_language(&input_path).map(|l| {
            println!("Auto-detected language: {}", l);
            l
        })
    }).expect("Could not determine language. Please specify with --lang");

    let final_diagram = if input_path.is_dir() {
        let mut stitcher = stitcher::Stitcher::new(input_path, lang);
        let mut directory = stitcher.build();
        directory.merge_all();
        directory.resolve_types(&stitcher.type_map);
        directory.merged_diagram
    } else {
        // Single file mode
        let mut parser = TreeSitterParser::new();
        match lang.to_lowercase().as_str() {
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
            "c#" | "cs" | "c-sharp" | "csharp" => {
                parser
                    .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                    .expect("Error loading c# grammar");
            }
            _ => {
                println!("Error {} is not a supported language", lang);
                std::process::exit(404);
            }
        }

        let source = std::fs::read(&args.source_code).expect("Failed to read source code file");
        let tree = parser.parse(&source, None).unwrap();
        let root_node = tree.root_node();
        let mut program_diagram = Diagram::new(&lang);
        program_diagram.build(root_node, &source);
        program_diagram
    };

    // pass to the exporter and write
    let _ = fs::write(args.destination, mermaid::generate(&final_diagram));
}
