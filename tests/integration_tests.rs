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

fn load_test_configs() -> Vec<LangTestConfig> {
    let mut configs = Vec::new();
    let langs = [
        "rust",
        "java",
        "javascript",
        "csharp",
        "cpp",
        "typescript",
        "objc",
        "dart",
    ];

    for lang in langs {
        let config_path = format!("languages/{}/test.yaml", lang);
        if let Ok(content) = fs::read_to_string(&config_path) {
            match serde_yml::from_str::<LangTestConfig>(&content) {
                Ok(config) => configs.push(config),
                Err(e) => eprintln!("Error parsing test config for {}: {}", lang, e),
            }
        } else {
            eprintln!("Warning: Could not find test config at {}", config_path);
        }
    }
    configs
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
        _ => panic!("Unsupported language: {}", lang),
    }
    parser
}

fn get_file_for_category(config: &LangTestConfig, category: &str) -> String {
    let file = config.files.get(category).expect(&format!(
        "No file defined for category {} in {}",
        category, config.name
    ));
    format!("{}/{}", config.path_prefix, file)
}

fn get_uml_for_category(config: &LangTestConfig, category: &str) -> String {
    let source_path = get_file_for_category(config, category);
    source_path
        .rsplit_once('.')
        .map(|(base, _)| format!("{}.uml", base))
        .expect(&format!("Source file has no extension: {}", source_path))
}

fn run_test(
    config: &LangTestConfig,
    category: &str,
    validator: impl FnOnce(&Diagram, &LangTestConfig),
) {
    let mut parser = setup_parser(&config.name);
    let path = get_file_for_category(config, category);
    let source = std::fs::read(&path).expect(&format!("failed to read test file: {}", path));
    let lang_config = LangConfig::load(&config.name);
    let mut diagram = Diagram::new(&lang_config);
    diagram.build(&source, &mut parser);

    let uml_path = get_uml_for_category(config, category);
    let expected_uml = std::fs::read_to_string(&uml_path)
        .expect(&format!("failed to read uml file: {}", uml_path));

    let generated_uml = generate(&diagram);

    assert_eq!(
        generated_uml.trim(),
        expected_uml.trim(),
        "Mermaid output mismatch for {} - {}",
        config.name,
        category
    );

    validator(&diagram, config);
}

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
        .find(|v| v.name == "latest_post")
        .unwrap();
    assert_eq!(latest_post_var.var_type, "models_Post");

    let current_user_var = app_class
        .variables
        .iter()
        .find(|v| v.name == "current_user")
        .unwrap();
    assert!(current_user_var.var_type == "models_User" || current_user_var.var_type == "auth_User");

    // Verify edge generation in Mermaid
    let output = generate(diagram);
    assert!(output.contains("App --> models_Post"));
}

#[test]
fn test_all_simple_structs() {
    let configs = load_test_configs();
    for config in configs {
        run_test(&config, "simple_struct", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "User")
                .expect(&format!("Class 'User' not found for {}", cfg.name));
            assert_eq!(
                class.variables.len(),
                3,
                "Variable count mismatch for {}",
                cfg.name
            );
        });
    }
}

#[test]
fn test_all_impl_blocks() {
    let configs = load_test_configs();
    for config in configs {
        run_test(&config, "impl_block", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "Calculator")
                .expect(&format!("Class 'Calculator' not found for {}", cfg.name));
            assert!(
                class.functions.len() >= 2,
                "Function count low for {}",
                cfg.name
            );
            assert!(class.functions.iter().any(|f| f.name == "add"));
        });
    }
}

#[test]
fn test_all_complex_types() {
    let configs = load_test_configs();
    for config in configs {
        run_test(&config, "complex_types", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name == "ComplexData")
                .expect(&format!("Class 'ComplexData' not found for {}", cfg.name));
            assert!(class.variables.iter().any(|v| v.name == "raw_bytes"));
            assert!(class.functions.iter().any(|f| f.name == "process"));
        });
    }
}

#[test]
fn test_all_generics() {
    let configs = load_test_configs();
    for config in configs {
        run_test(&config, "generics", |diagram, cfg| {
            let class = diagram
                .classes
                .iter()
                .find(|c| c.name.contains("Box"))
                .expect(&format!("Generic class 'Box' not found for {}", cfg.name));
            assert!(class.variables.iter().any(|v| v.name == "inner"));
        });
    }
}

#[test]
fn test_mermaid_smoke() {
    let configs = load_test_configs();
    let rust_config = configs
        .iter()
        .find(|c| c.name == "rust")
        .expect("Rust config not found");
    // Just verify it doesn't crash and produces basic output for the first language
    run_test(rust_config, "simple_struct", |diagram, _| {
        let output = generate(diagram);
        assert!(output.contains("classDiagram"));
        assert!(output.contains("User"));
    });
}
