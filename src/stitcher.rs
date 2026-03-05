use crate::diagram::Diagram;
use crate::lang_config::LangConfig;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::Parser as TreeSitterParser;

pub struct File {
    pub diagram: Diagram,
}

pub struct Directory {
    pub sub_directories: Vec<Directory>,
    pub files: Vec<File>,
    pub merged_diagram: Diagram,
}

impl Directory {
    pub fn new(lang: LangConfig) -> Self {
        Directory {
            sub_directories: Vec::new(),
            files: Vec::new(),
            merged_diagram: Diagram::new(lang),
        }
    }

    /// Recursively merge all diagrams into the root directory's merged_diagram
    pub fn merge_all(&mut self) {
        // First, merge subdirectories recursively
        for sub_dir in &mut self.sub_directories {
            sub_dir.merge_all();
            let mut classes = sub_dir.merged_diagram.classes.drain(..).collect();
            self.merged_diagram.classes.append(&mut classes);
            let mut imports = sub_dir.merged_diagram.imports.drain(..).collect();
            self.merged_diagram.imports.append(&mut imports);
        }

        // Then merge files in current directory
        for file in &mut self.files {
            let mut classes = file.diagram.classes.drain(..).collect();
            self.merged_diagram.classes.append(&mut classes);
            let mut imports = file.diagram.imports.drain(..).collect();
            self.merged_diagram.imports.append(&mut imports);
        }
    }

    /// Resolve types across all classes in the merged diagram
    pub fn resolve_types(&mut self, type_map: &GlobalTypeMap) {
        for class in &mut self.merged_diagram.classes {
            // Resolve variable types
            for var in &mut class.variables {
                if let Some(qualified) =
                    type_map.resolve(&var.var_type, &class.name, &self.merged_diagram.imports)
                {
                    var.var_type = qualified;
                }
                for inner in &mut var.inner_types {
                    if let Some(qualified) =
                        type_map.resolve(inner, &class.name, &self.merged_diagram.imports)
                    {
                        *inner = qualified;
                    }
                }
            }
            // Resolve function return and argument types
            for func in &mut class.functions {
                if let Some(qualified) =
                    type_map.resolve(&func.return_type, &class.name, &self.merged_diagram.imports)
                {
                    func.return_type = qualified;
                }
                for inner in &mut func.return_inner_types {
                    if let Some(qualified) =
                        type_map.resolve(inner, &class.name, &self.merged_diagram.imports)
                    {
                        *inner = qualified;
                    }
                }
                for arg in &mut func.arguments {
                    if let Some(qualified) =
                        type_map.resolve(&arg.var_type, &class.name, &self.merged_diagram.imports)
                    {
                        arg.var_type = qualified;
                    }
                    for inner in &mut arg.inner_types {
                        if let Some(qualified) =
                            type_map.resolve(inner, &class.name, &self.merged_diagram.imports)
                        {
                            *inner = qualified;
                        }
                    }
                }
            }
        }
    }
}

pub struct GlobalTypeMap {
    /// Maps short class name to a list of fully qualified names (namespaced)
    pub types: HashMap<String, Vec<String>>,
}

impl GlobalTypeMap {
    pub fn new() -> Self {
        GlobalTypeMap {
            types: HashMap::new(),
        }
    }

    pub fn insert(&mut self, short_name: String, qualified_name: String) {
        self.types
            .entry(short_name)
            .or_insert_with(Vec::new)
            .push(qualified_name);
    }

    /// Resolution heuristic for a type T used in a class C
    pub fn resolve(
        &self,
        type_name: &str,
        current_class_qualified: &str,
        _imports: &[String],
    ) -> Option<String> {
        let candidates = self.types.get(type_name)?;

        // 1. If only one candidate, it's likely the one (simplification for now)
        if candidates.len() == 1 {
            return Some(candidates[0].clone());
        }

        // 2. Check for same-file/same-namespace (heuristic: prefix match)
        // Find the candidate with the longest common prefix with the current class
        let mut best_candidate = None;
        let mut max_prefix_match = 0;

        for candidate in candidates {
            let common_prefix_len = current_class_qualified
                .chars()
                .zip(candidate.chars())
                .take_while(|(a, b)| a == b)
                .count();

            if common_prefix_len > max_prefix_match {
                max_prefix_match = common_prefix_len;
                best_candidate = Some(candidate.clone());
            }
        }

        best_candidate.or_else(|| Some(candidates[0].clone()))
    }
}

pub struct Stitcher {
    pub root_path: PathBuf,
    pub type_map: GlobalTypeMap,
    pub config: crate::lang_config::LangConfig,
    pub parser: TreeSitterParser,
}

impl Stitcher {
    pub fn new(root_path: PathBuf, config: LangConfig, parser: TreeSitterParser) -> Self {
        Stitcher {
            root_path,
            type_map: GlobalTypeMap::new(),
            config,
            parser,
        }
    }

    pub fn build(&mut self) -> Directory {
        let mut root_dir = Directory::new(self.config.clone());
        self.process_directory(&self.root_path.clone(), &mut root_dir);
        root_dir
    }

    fn process_directory(&mut self, current_path: &Path, current_dir: &mut Directory) {
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let mut sub_dir = Directory::new(self.config.clone());
                    self.process_directory(&path, &mut sub_dir);
                    current_dir.sub_directories.push(sub_dir);
                } else if self.is_source_file(&path) {
                    if let Some(file_node) = self.process_file(&path) {
                        current_dir.files.push(file_node);
                    }
                }
            }
        }
    }

    fn is_source_file(&self, path: &Path) -> bool {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        self.config
            .file_extensions
            .iter()
            .any(|ext| ext == extension)
    }

    fn process_file(&mut self, path: &Path) -> Option<File> {
        let source = fs::read(path).ok()?;

        let mut diagram = Diagram::new(self.config.clone());
        diagram.build(&source, &mut self.parser);

        let relative_path = path
            .strip_prefix(&self.root_path)
            .unwrap_or(path)
            .to_path_buf();

        // Qualify class names based on path to prevent collisions
        let path_prefix = relative_path
            .parent()
            .map(|p| {
                p.to_string_lossy()
                    .to_string()
                    .replace(['/', '\\', '.'], "_")
            })
            .unwrap_or_default();

        for class in &mut diagram.classes {
            let original_name = class.name.clone();
            if !path_prefix.is_empty() {
                class.name = format!("{}_{}", path_prefix, class.name);
            }
            // If class already has a namespace, prepend that too
            if !class.namespace.is_empty() {
                class.name = format!("{}_{}", class.namespace, class.name);
            }

            self.type_map.insert(original_name, class.name.clone());
        }

        Some(File { diagram })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tree_sitter::Parser;

    fn setup_parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        parser
    }

    #[test]
    fn test_is_source_file() {
        let rust_config = LangConfig::load("rust");
        let stitcher = Stitcher::new(PathBuf::from("."), rust_config, setup_parser());
        assert!(stitcher.is_source_file(Path::new("test.rs")));
        assert!(!stitcher.is_source_file(Path::new("test.txt")));

        let java_config = LangConfig::load("java");
        let mut java_parser = Parser::new();
        java_parser.set_language(&tree_sitter_java::LANGUAGE.into()).unwrap();
        let stitcher_java = Stitcher::new(PathBuf::from("."), java_config, java_parser);
        assert!(stitcher_java.is_source_file(Path::new("Test.java")));
    }

    #[test]
    fn test_type_resolution_heuristic() {
        let mut map = GlobalTypeMap::new();
        map.insert("User".to_string(), "models_User".to_string());
        map.insert("User".to_string(), "auth_User".to_string());

        // Resolve User for a class in models_... should prefer models_User
        let resolved = map.resolve("User", "models_Account", &[]);
        assert_eq!(resolved.unwrap(), "models_User");

        // Resolve User for a class in auth_... should prefer auth_User
        let resolved_auth = map.resolve("User", "auth_Session", &[]);
        assert_eq!(resolved_auth.unwrap(), "auth_User");

        // Resolve something that has only one candidate
        map.insert("Database".to_string(), "core_Database".to_string());
        let resolved_db = map.resolve("Database", "auth_Session", &[]);
        assert_eq!(resolved_db.unwrap(), "core_Database");
    }

    #[test]
    fn test_stitcher_build_real_dir() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("languages");
        root.push("rust");

        let rust_config = LangConfig::load("rust");
        let mut stitcher = Stitcher::new(root, rust_config, setup_parser());
        let mut directory = stitcher.build();
        directory.merge_all();
        directory.resolve_types(&stitcher.type_map);

        let class_names: Vec<String> = directory
            .merged_diagram
            .classes
            .iter()
            .map(|c| c.name.clone())
            .collect();

        println!("Found classes: {:?}", class_names);

        assert!(class_names.iter().any(|name| name.contains("ComplexData")));
        assert!(class_names.iter().any(|name| name.contains("User")));
        assert!(class_names.iter().any(|name| name.contains("Calculator")));

        // Ensure type map was populated
        assert!(stitcher.type_map.types.contains_key("ComplexData"));
        assert!(stitcher.type_map.types.contains_key("User"));
        assert!(stitcher.type_map.types.contains_key("Calculator"));
    }
}
