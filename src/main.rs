mod diagram;
mod lang_config;
mod mermaid;
mod stitcher;
use crate::diagram::Diagram;
use crate::lang_config::LangConfig;
use clap::{ArgGroup, Parser};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tree_sitter::Parser as TreeSitterParser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(ArgGroup::new("source").args(&["source_code", "git"])))]
struct Args {
    /// List available programming languages
    #[arg(long, action = clap::ArgAction::SetTrue)]
    list_languages: bool,

    /// Programming language (optional, auto-detected if omitted)
    #[arg(short, long)]
    lang: Option<String>,
    /// Path to the local source file or directory
    #[arg(short, long, conflicts_with = "git", default_value("."))]
    source_code: String,
    /// Remote git repository URL to clone and analyze
    #[arg(long, conflicts_with = "source_code")]
    git: Option<String>,

    /// Outputs the computed mermaid diagram without surrounding markdown code block
    #[arg(long, action = clap::ArgAction::SetTrue)]
    no_mermaid: bool,

    /// Write destination file for the exporter
    #[arg(short, long, default_value("UML.md"))]
    destination: String,
}

fn detect_language(path: &std::path::Path) -> Option<String> {
    if path.is_file() {
        let ext = path.extension()?.to_str()?.to_lowercase();

        // Check each embedded language config
        for (lang_name, config) in crate::lang_config::LangConfig::all_configs() {
            if config.file_extensions.iter().any(|e| e == &ext) {
                return Some(lang_name);
            }
        }
        return None;
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

fn get_parser(lang: &str) -> TreeSitterParser {
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
                .set_language(&tree_sitter_dart::language())
                .expect("Error loading dart grammar");
        }
        _ => {
            println!("Error {} is not a supported language", lang);
            std::process::exit(404);
        }
    }
    parser
}

fn main() {
    let args = Args::parse();

    // list languages and exit early
    if args.list_languages {
        println!("Available programming languages:");
        for lang in LangConfig::list_languages() {
            println!(" - {}", lang);
        }
        return;
    }

    // Handle the optional remote git directory
    let temp_dir: Option<std::path::PathBuf>;
    let input_path = if let Some(git_url) = &args.git {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_path = std::env::temp_dir()
            .join("auto-uml")
            .join(format!("-cloned-{}", timestamp));

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
        // handle simple local code path
        temp_dir = None;
        std::path::PathBuf::from(args.source_code)
    };

    // define language or if flag not set attempt to detect language
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

    let config: LangConfig = LangConfig::load(&lang);
    let mut parser = get_parser(&lang);

    // create diagram
    let final_diagram = if input_path.is_dir() {
        let mut stitcher = stitcher::Stitcher::new(input_path, &config, parser);
        let mut directory = stitcher.build();
        directory.merge_all();
        directory.resolve_types(&stitcher.type_map);
        directory.merged_diagram
    } else {
        // Single file mode
        let source = std::fs::read(&input_path).expect("Failed to read source code file");
        let mut program_diagram = Diagram::new(&config);
        program_diagram.build(&source, &mut parser);
        program_diagram
    };

    let mermaid: String = match args.no_mermaid {
        true => mermaid::generate(&final_diagram),
        false => mermaid::generate_code_block(&final_diagram),
    };

    // pass to the exporter and write
    fs::write(&args.destination, mermaid).expect("Failed to write to destination file");
    println!("Diagram written to {}", args.destination);

    // Clean up temp directory if we cloned from git
    if let Some(temp_path) = temp_dir {
        println!("Cleaning up temporary directory...");
        std::fs::remove_dir_all(&temp_path).ok();
    }
}
