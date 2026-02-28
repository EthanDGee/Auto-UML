use crate::diagram::Diagram;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::Parser as TreeSitterParser;

pub struct File {
    pub name: String,
    pub relative_path: PathBuf,
    pub diagram: Diagram,
}

pub struct Directory {
    pub name: String,
    pub sub_directories: Vec<Directory>,
    pub files: Vec<File>,
    pub merged_diagram: Diagram,
}

impl Directory {
    pub fn new(name: String, lang: &str) -> Self {
        Directory {
            name,
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
}

pub struct Stitcher {
    pub root_path: PathBuf,
    pub language: String,
    pub type_map: GlobalTypeMap,
}

impl Stitcher {
    pub fn new(root_path: PathBuf, language: String) -> Self {
        Stitcher {
            root_path,
            language,
            type_map: GlobalTypeMap::new(),
        }
    }

    pub fn build(&mut self) -> Directory {
        let root_name = self
            .root_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut root_dir = Directory::new(root_name, &self.language);
        self.process_directory(&self.root_path.clone(), &mut root_dir);
        root_dir
    }

    fn process_directory(&mut self, current_path: &Path, current_dir: &mut Directory) {
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let mut sub_dir = Directory::new(dir_name, &self.language);
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
        match self.language.to_lowercase().as_str() {
            "rust" => extension == "rs",
            "java" => extension == "java",
            "javascript" | "js" => extension == "js",
            "typescript" | "ts" => extension == "ts" || extension == "tsx",
            "cpp" | "c++" => extension == "cpp" || extension == "h" || extension == "hpp" || extension == "cc",
            "csharp" | "cs" | "c-sharp" => extension == "cs",
            _ => false,
        }
    }

    fn process_file(&mut self, path: &Path) -> Option<File> {
        let source = fs::read(path).ok()?;
        let mut parser = TreeSitterParser::new();

        match self.language.to_lowercase().as_str() {
            "rust" => {
                parser
                    .set_language(&tree_sitter_rust::LANGUAGE.into())
                    .ok()?;
            }
            "java" => {
                parser
                    .set_language(&tree_sitter_java::LANGUAGE.into())
                    .ok()?;
            }
            "js" | "javascript" => {
                parser
                    .set_language(&tree_sitter_javascript::LANGUAGE.into())
                    .ok()?;
            }
            "ts" | "typescript" => {
                parser
                    .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                    .ok()?;
            }
            "c++" | "cpp" => {
                parser
                    .set_language(&tree_sitter_cpp::LANGUAGE.into())
                    .ok()?;
            }
            "c#" | "cs" | "c-sharp" => {
                parser
                    .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                    .ok()?;
            }
            _ => return None,
        }

        let tree = parser.parse(&source, None)?;
        let mut diagram = Diagram::new(&self.language);
        diagram.build(tree.root_node(), &source);

        let relative_path = path
            .strip_prefix(&self.root_path)
            .unwrap_or(path)
            .to_path_buf();

        // Qualify class names based on path to prevent collisions
        let path_prefix = relative_path
            .parent()
            .map(|p| p.to_string_lossy().to_string().replace(['/', '\\', '.'], "_"))
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

        Some(File {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            relative_path,
            diagram,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_source_file() {
        let stitcher = Stitcher::new(PathBuf::from("."), "rust".to_string());
        assert!(stitcher.is_source_file(Path::new("test.rs")));
        assert!(!stitcher.is_source_file(Path::new("test.txt")));

        let stitcher_java = Stitcher::new(PathBuf::from("."), "java".to_string());
        assert!(stitcher_java.is_source_file(Path::new("Test.java")));
    }

    #[test]
    fn test_global_type_map() {
        let mut map = GlobalTypeMap::new();
        map.insert("User".to_string(), "models_User".to_string());
        map.insert("User".to_string(), "auth_User".to_string());

        assert_eq!(map.types.get("User").unwrap().len(), 2);
        assert!(map.types.get("User").unwrap().contains(&"models_User".to_string()));
        assert!(map.types.get("User").unwrap().contains(&"auth_User".to_string()));
    }

    #[test]
    fn test_stitcher_build_real_dir() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("test_source_code_examples");
        root.push("rust");

        let mut stitcher = Stitcher::new(root, "rust".to_string());
        let mut directory = stitcher.build();
        directory.merge_all();

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
