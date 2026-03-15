use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::path::Path;

static LANG_DIR: Dir<'_> = include_dir!("$STAGED_LANGS_DIR");

#[derive(Clone, Deserialize, Debug, Default)]
pub struct LangConfig {
    pub file_extensions: Vec<String>,
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
    pub visibility_modifier_patterns: Vec<String>,
    pub private_by_default: bool,
    pub public_modifier_patterns: Vec<String>,
    pub private_modifier_patterns: Vec<String>,
}

impl LangConfig {
    pub fn all_configs() -> Vec<(String, Self)> {
        let mut configs = Vec::new();
        for dir in LANG_DIR.dirs() {
            if let Some(lang_name) = dir.path().file_name().and_then(|n| n.to_str()) {
                let config = Self::load(lang_name);
                configs.push((lang_name.to_string(), config));
            }
        }
        configs
    }

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

        let config_path = Path::new(lang_dir).join("config.yaml");

        if let Some(file) = LANG_DIR.get_file(config_path) {
            if let Some(content) = file.contents_utf8() {
                match serde_yml::from_str::<LangConfig>(content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Error parsing embedded config for {}: {}", language, e);
                    }
                }
            }
        } else {
            eprintln!(
                "Warning: Could not find embedded config for language: {}",
                language
            );
        }

        // Return a default empty config if loading fails
        LangConfig::default()
    }

    pub fn list_languages() -> Vec<&'static str> {
        vec![
            "rust",
            "java",
            "javascript",
            "typescript",
            "cpp",
            "csharp",
            "objective-c",
            "dart",
        ]
    }
}
