use include_dir::{Dir, include_dir};
use serde::Deserialize;
use std::path::Path;

static LANG_DIR: Dir<'_> = include_dir!("$STAGED_LANGS_DIR");

/// Per-language analyzer configuration.
///
/// Every field is optional: a missing key in `config.yaml` deserializes to `None` (serde
/// special-cases `Option<T>`). The analyzer treats `None` and an empty value identically — it
/// simply skips the corresponding operation. So a config only needs to declare the capabilities
/// the language actually uses.
#[derive(Clone, Deserialize, Debug, Default)]
pub struct LangConfig {
    pub file_extensions: Option<Vec<String>>,
    pub class_patterns: Option<Vec<String>>,
    pub function_patterns: Option<Vec<String>>,
    pub variable_patterns: Option<Vec<String>>,
    pub identifier_patterns: Option<Vec<String>>,
    pub type_patterns: Option<Vec<String>>,
    pub parameter_container_patterns: Option<Vec<String>>,
    pub parameter_patterns: Option<Vec<String>>,
    pub wrapper_patterns: Option<Vec<String>>,
    pub skip_patterns: Option<Vec<String>>,
    pub import_patterns: Option<Vec<String>>,
    pub namespace_patterns: Option<Vec<String>>,
    pub visibility_modifier_patterns: Option<Vec<String>>,
    pub private_by_default: Option<bool>,
    pub public_modifier_patterns: Option<Vec<String>>,
    pub private_modifier_patterns: Option<Vec<String>>,
    pub class_type_parameter_patterns: Option<Vec<String>>,
    pub type_path_separator: Option<String>,
    pub self_parameter_patterns: Option<Vec<String>>,
    pub type_annotation_strip_prefix: Option<String>,
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
            "kotlin" | "kt" => "kotlin",
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
            "kotlin",
        ]
    }
}
