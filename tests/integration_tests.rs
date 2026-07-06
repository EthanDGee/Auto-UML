use auto_uml::diagram::Diagram;
use auto_uml::lang_config::LangConfig;
use auto_uml::mermaid::generate;
use auto_uml::stitcher::Stitcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tree_sitter::Parser;

/// Configuration for a specific language's integration tests.
#[derive(Deserialize, Debug)]
struct LangTestConfig {
    name: String,
    path_prefix: String,
    /// Mapping of (Test Category) -> (File Name)
    files: HashMap<String, String>,
}

fn load_single_lang_config(lang_dir: &str) -> LangTestConfig {
    let config_path = format!("languages/{}/test.yaml", lang_dir);
    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| panic!("Could not find test config at {}", config_path));
    serde_yml::from_str::<LangTestConfig>(&content)
        .unwrap_or_else(|e| panic!("Error parsing test config for {}: {}", lang_dir, e))
}

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
        "javascript" => {
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .expect("Error loading JavaScript grammar");
        }
        "csharp" => {
            parser
                .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                .expect("Error loading C# grammar");
        }
        "cpp" => {
            parser
                .set_language(&tree_sitter_cpp::LANGUAGE.into())
                .expect("Error loading C++ grammar");
        }
        "typescript" => {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .expect("Error loading TypeScript grammar");
        }
        "objective-c" => {
            parser
                .set_language(&tree_sitter_objc::LANGUAGE.into())
                .expect("Error loading Objective-C grammar");
        }
        "dart" => {
            parser
                .set_language(&tree_sitter_dart::language())
                .expect("Error loading Dart grammar");
        }
        "kotlin" => {
            parser
                .set_language(&tree_sitter_kotlin::LANGUAGE.into())
                .expect("Error loading Kotlin grammar");
        }
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}

fn get_file_for_category(config: &LangTestConfig, category: &str) -> String {
    let file = config
        .files
        .get(category)
        .expect("No file defined for category");
    format!("{}/{}", config.path_prefix, file)
}

fn get_uml_for_category(config: &LangTestConfig, category: &str) -> String {
    let source_path = get_file_for_category(config, category);
    source_path
        .rsplit_once('.')
        .map(|(base, _)| format!("{}.uml", base))
        .expect("Source file has no extension")
}

fn run_test(
    config: &LangTestConfig,
    category: &str,
    validator: impl FnOnce(&Diagram, &LangTestConfig),
) {
    let mut parser = setup_parser(&config.name);
    let path = get_file_for_category(config, category);
    let source = std::fs::read(&path).expect("failed to read test file");
    let lang_config = LangConfig::load(&config.name);
    let mut diagram = Diagram::new(&lang_config);
    diagram.build(&source, &mut parser);

    let uml_path = get_uml_for_category(config, category);
    let expected_uml = std::fs::read_to_string(&uml_path).expect("failed to read uml file");

    let generated_uml = generate(&diagram);
    let generated_uml = generated_uml.trim();
    let expected_uml = expected_uml.trim();

    if generated_uml != expected_uml {
        eprintln!(
            "\n\x1b[1;31m--- Mermaid Mismatch for {} - {} ---\x1b[0m",
            config.name, category
        );
        eprintln!("\x1b[1;32m+++ Expected:\x1b[0m\n{}", expected_uml);
        eprintln!("\x1b[1;31m--- Generated:\x1b[0m\n{}", generated_uml);
        eprintln!("\x1b[1;34m-------------------------------------------\x1b[0m\n");
    }

    pretty_assertions::assert_eq!(
        generated_uml,
        expected_uml,
        "Mermaid output mismatch for {} - {}",
        config.name,
        category
    );

    validator(&diagram, config);
}

macro_rules! lang_tests {
    ($mod_name:ident, $lang_dir:literal) => {
        mod $mod_name {
            fn config() -> super::LangTestConfig {
                super::load_single_lang_config($lang_dir)
            }

            #[test]
            fn simple_struct() {
                let cfg = config();
                super::run_test(&cfg, "simple_struct", |diagram, c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "User")
                        .expect("Class 'User' not found");
                    assert_eq!(
                        class.variables.len(),
                        3,
                        "Variable count mismatch for {}",
                        c.name
                    );
                    let output = super::generate(diagram);
                    assert!(output.contains("classDiagram"));
                    assert!(output.contains("User"));
                });
            }

            #[test]
            fn impl_block() {
                let cfg = config();
                super::run_test(&cfg, "impl_block", |diagram, c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "Calculator")
                        .expect("Class 'Calculator' not found");
                    assert!(
                        class.functions.len() >= 2,
                        "Function count low for {}",
                        c.name
                    );
                    assert!(class.functions.iter().any(|f| f.name == "add"));
                });
            }

            #[test]
            fn complex_types() {
                let cfg = config();
                super::run_test(&cfg, "complex_types", |diagram, _c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name == "ComplexData")
                        .expect("Class 'ComplexData' not found");
                    assert!(
                        class
                            .variables
                            .iter()
                            .any(|v| v.name.as_deref() == Some("raw_bytes"))
                    );
                    assert!(class.functions.iter().any(|f| f.name == "process"));
                });
            }

            #[test]
            fn generics() {
                let cfg = config();
                super::run_test(&cfg, "generics", |diagram, _c| {
                    let class = diagram
                        .classes
                        .iter()
                        .find(|cl| cl.name.contains("Box"))
                        .expect("Generic class 'Box' not found");
                    assert!(
                        class
                            .variables
                            .iter()
                            .any(|v| v.name.as_deref() == Some("inner"))
                    );
                });
            }
        }
    };
}

lang_tests!(rust, "rust");
lang_tests!(java, "java");
lang_tests!(javascript, "javascript");
lang_tests!(csharp, "csharp");
lang_tests!(cpp, "cpp");
lang_tests!(typescript, "typescript");
lang_tests!(objc, "objc");
lang_tests!(dart, "dart");
lang_tests!(kotlin, "kotlin");

#[test]
fn test_stitcher_integration() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("languages");
    root.push("stitch_test");

    let lang_config = LangConfig::load("rust");
    let mut stitcher = Stitcher::new(root, &lang_config, setup_parser("rust"));
    let mut directory = stitcher.build();
    directory.merge_all();
    directory.resolve_types(&stitcher.type_map);

    let diagram = &directory.merged_diagram;

    // Verify all classes were found with qualified names
    // Path-based qualification: models_User, models_Post, auth_User, App
    let class_names: Vec<String> = diagram.classes.iter().map(|c| c.name.clone()).collect();

    assert!(class_names.contains(&"models_User".to_string()));
    assert!(class_names.contains(&"models_Post".to_string()));
    assert!(class_names.contains(&"auth_User".to_string()));
    assert!(class_names.contains(&"App".to_string()));

    // Verify type resolution in 'App'
    let app_class = diagram.classes.iter().find(|c| c.name == "App").unwrap();

    // latest_post: Post -> resolved to models_Post
    let latest_post_var = app_class
        .variables
        .iter()
        .find(|v| v.name.as_deref() == Some("latest_post"))
        .unwrap();
    assert_eq!(latest_post_var.var_type, "models_Post");

    let current_user_var = app_class
        .variables
        .iter()
        .find(|v| v.name.as_deref() == Some("current_user"))
        .unwrap();
    assert!(current_user_var.var_type == "models_User" || current_user_var.var_type == "auth_User");

    // Verify edge generation in Mermaid
    let output = generate(diagram);
    assert!(output.contains("App --> models_Post"));
}
