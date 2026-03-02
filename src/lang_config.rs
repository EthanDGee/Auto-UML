use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Deserialize, Debug, Default)]
pub struct LangConfig {
    pub class_patterns: Vec<String>,
    pub function_patterns: Vec<String>,
    pub variable_patterns: Vec<String>,
    pub identifier_patterns: Vec<String>,
    pub type_patterns: Vec<String>,
    pub parameter_container_patterns: Vec<String>,
    pub parameter_patterns: Vec<String>,
    pub wrapper_patterns: Vec<String>,
    pub skip_patterns: Vec<String>,
    pub import_patterns: Vec<String>,
    pub namespace_patterns: Vec<String>,
}

impl LangConfig {
    pub fn load(language: &str) -> Self {
        let lang_dir = match language.to_lowercase().as_str() {
            "rust" => "rust",
            "java" => "java",
            "javascript" | "js" => "javascript",
            "typescript" | "ts" => "typescript",
            "cpp" | "c++" => "cpp",
            "csharp" | "cs" | "c-sharp" => "csharp",
            "objective-c" | "objc" => "objc",
            "dart" => "dart",
            _ => language,
        };

        let config_path = Path::new("languages").join(lang_dir).join("config.yaml");

        if let Ok(content) = fs::read_to_string(&config_path) {
            match serde_yml::from_str::<LangConfig>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Error parsing config for {}: {}", language, e);
                }
            }
        } else {
            eprintln!("Warning: Could not find config at {:?}", config_path);
        }

        // Return a default empty config if loading fails
        LangConfig::default()
    }
}
