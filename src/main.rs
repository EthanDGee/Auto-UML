use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tree_sitter::Parser as TreeSitterParser;
mod diagram;
mod lang_config;
mod mermaid;
mod stitcher;
use clap::{ArgGroup, Parser};

use crate::diagram::Diagram;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(ArgGroup::new("source").required(true).args(&["source_code", "git"])))]
struct Args {
    /// Programming language (optional, auto-detected if omitted)
    #[arg(short, long)]
    lang: Option<String>,
    /// Path to the local source file or directory
    #[arg(short, long, conflicts_with = "git")]
    source_code: Option<String>,
    /// Remote git repository URL to clone and analyze
    #[arg(long, conflicts_with = "source_code")]
    git: Option<String>,

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
            "m" => Some("objective-c".to_string()),
            "dart" => Some("dart".to_string()),
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

    let temp_dir: Option<std::path::PathBuf>;
    let input_path = if let Some(git_url) = &args.git {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let rand_suffix: u16 = rand_simple();
        let temp_path = std::env::temp_dir()
            .join("auto-uml")
            .join(format!("cloned-{}-{}", timestamp, rand_suffix));

        if let Some(parent) = temp_path.parent() {
            std::fs::create_dir_all(parent)
                .expect("Failed to create temporary directory for clone");
        }

        println!("Cloning {} to {}...", git_url, temp_path.display());

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.transfer_progress(|progress| {
            print!(
                "\rCloning: {}/{} objects ({}%)",
                progress.received_objects(),
                progress.total_objects(),
                100 * progress.received_objects() / progress.total_objects()
            );
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            true
        });

        let mut options = git2::FetchOptions::new();
        options.remote_callbacks(callbacks);

        git2::Repository::clone(git_url, &temp_path).expect("Failed to clone repository");

        println!("\nClone complete. Analyzing...");

        temp_dir = Some(temp_path.clone());
        temp_path
    } else {
        temp_dir = None;
        std::path::PathBuf::from(args.source_code.as_ref().expect("source_code required"))
    };

    let lang = args
        .lang
        .clone()
        .or_else(|| {
            detect_language(&input_path).map(|l| {
                println!("Auto-detected language: {}", l);
                l
            })
        })
        .expect("Could not determine language. Please specify with --lang");

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
            "objective-c" | "objc" => {
                parser
                    .set_language(&tree_sitter_objc::LANGUAGE.into())
                    .expect("Error loading objective-c grammar");
            }
            "dart" => {
                parser
                    .set_language(&tree_sitter_dart::language().into())
                    .expect("Error loading dart grammar");
            }
            _ => {
                println!("Error {} is not a supported language", lang);
                std::process::exit(404);
            }
        }

        let source = std::fs::read(&input_path).expect("Failed to read source code file");
        let tree = parser.parse(&source, None).unwrap();
        let root_node = tree.root_node();
        let mut program_diagram = Diagram::new(&lang);
        program_diagram.build(root_node, &source);
        program_diagram
    };

    // pass to the exporter and write
    fs::write(&args.destination, mermaid::generate(&final_diagram))
        .expect("Failed to write to destination file");
    println!("Diagram written to {}", args.destination);

    // Clean up temp directory if we cloned from git
    if let Some(temp_path) = temp_dir {
        println!("Cleaning up temporary directory...");
        std::fs::remove_dir_all(&temp_path).ok();
    }
}

fn rand_simple() -> u16 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u8(
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u8,
    );
    (hasher.finish() % 65536) as u16
}
