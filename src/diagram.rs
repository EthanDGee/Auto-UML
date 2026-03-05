use crate::lang_config::LangConfig;
use tree_sitter::Node;

pub struct Variable {
    pub name: String,
    pub var_type: String,
    pub inner_types: Vec<String>,
}

impl Variable {
    pub fn new(name: String, var_type: String, inner_types: Vec<String>) -> Self {
        Variable {
            name,
            var_type,
            inner_types,
        }
    }
}

pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub return_type: String,
    pub return_inner_types: Vec<String>,
}

impl Function {
    pub fn new(name: String, return_type: String, return_inner_types: Vec<String>) -> Self {
        Function {
            name,
            arguments: Vec::new(),
            return_type,
            return_inner_types,
        }
    }

    pub fn add_argument(&mut self, arg: Variable) {
        self.arguments.push(arg);
    }
}

pub struct Class {
    pub name: String,
    pub namespace: String,
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
}

impl Class {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Class {
            name,
            namespace: String::new(),
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn with_namespace(name: String, namespace: String) -> Self {
        Class {
            name,
            namespace,
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }

    pub fn add_variable(&mut self, var: Variable) {
        self.variables.push(var);
    }
}

pub struct Diagram {
    pub classes: Vec<Class>,
    pub imports: Vec<String>,
    lang: LangConfig,
}

impl Diagram {
    pub fn new(lang: LangConfig) -> Self {
        Diagram {
            classes: Vec::new(),
            imports: Vec::new(),
            lang,
        }
    }

    pub fn build(&mut self, source: &[u8], parser: &mut tree_sitter::Parser) {
        let tree = parser.parse(source, None).unwrap();
        self.navigate_node(tree.root_node(), source, None, "");
    }

    /// Recursively navigate the tree_sitter tree and build out Diagram
    pub fn navigate_node(
        &mut self,
        node: Node,
        source: &[u8],
        class_index: Option<usize>,
        current_namespace: &str,
    ) {
        let kind = node.kind();
        let mut next_class_index = class_index;
        let mut active_namespace = current_namespace.to_string();

        if self.lang.import_patterns.iter().any(|p| p == kind) {
            let import_text =
                String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]).to_string();
            self.imports.push(import_text);
        } else if self.lang.namespace_patterns.iter().any(|p| p == kind) {
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                if active_namespace.is_empty() {
                    active_namespace = name;
                } else {
                    active_namespace = format!("{}_{}", active_namespace, name);
                }
            }
        } else if self.lang.class_patterns.iter().any(|p| p == kind) {
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                // update class index to match preexisting class if already exist
                if let Some(idx) = self
                    .classes
                    .iter()
                    .position(|class| class.name == name && class.namespace == active_namespace)
                {
                    next_class_index = Some(idx);
                } else {
                    // create new class and update indexes
                    self.classes
                        .push(Class::with_namespace(name, active_namespace.clone()));
                    next_class_index = Some(self.classes.len() - 1);
                }
            }
        } else if self.lang.function_patterns.iter().any(|p| p == kind) {
            // Function/Method detection
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                let types = self.extract_type(node, source);
                let main_type = types.first().cloned().unwrap_or_else(|| "void".to_string());
                let inners = if types.len() > 1 {
                    types[1..].to_vec()
                } else {
                    Vec::new()
                };
                let mut func = Function::new(name, main_type, inners);
                self.extract_parameters(node, source, &mut func);

                if let Some(idx) = next_class_index {
                    self.classes[idx].add_function(func);
                }
            }
        } else if self.lang.variable_patterns.iter().any(|p| p == kind) {
            // Field/Variable detection
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                let types = self.extract_type(node, source);
                let main_type = types.first().cloned().unwrap_or_else(|| "void".to_string());
                let inners = if types.len() > 1 {
                    types[1..].to_vec()
                } else {
                    Vec::new()
                };
                let var = Variable::new(name, main_type, inners);
                if let Some(idx) = next_class_index {
                    self.classes[idx].add_variable(var);
                }
            }
        }

        // Recursively travel all children nodes (break case is handled by empty for loop)
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            self.navigate_node(child, source, next_class_index, &active_namespace);
        }
    }

    /// Helper to find identifiers (names) which may have different kind names across grammars.
    fn extract_identifier(&self, node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        let mut best_guess = String::new();

        for child in node.children(&mut cursor) {
            let kind = child.kind();

            if self.lang.skip_patterns.iter().any(|p| p == kind) {
                continue;
            }
            if self.lang.identifier_patterns.iter().any(|p| p == kind) {
                if kind == "identifier" || kind == "field_identifier" {
                    return String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                        .to_string();
                }
                if best_guess.is_empty() {
                    best_guess =
                        String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                            .to_string();
                }
            }

            // Recurse into certain nodes that wrap identifiers
            if self.lang.wrapper_patterns.iter().any(|p| p == kind) {
                let name = self.extract_identifier(child, source);
                if !name.is_empty() {
                    return name;
                }
            }
        }
        best_guess
    }

    /// Helper to extract type information from a node.
    fn extract_type(&self, node: Node, source: &[u8]) -> Vec<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();

            if self
                .lang
                .type_patterns
                .iter()
                .any(|p| kind == p || (p == "type" && kind.contains("type")))
            {
                let full_type =
                    String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                        .to_string();

                // Naive parsing of generics: "Vec<User>" -> ["Vec", "User"]
                if let Some(pos) = full_type.find('<') {
                    let main = full_type[..pos].trim().to_string();
                    let mut inners = Vec::new();
                    if let Some(end_pos) = full_type.rfind('>') {
                        let inner_str = &full_type[pos + 1..end_pos];
                        // split by comma for multiple generics like HashMap<K, V>
                        for part in inner_str.split(',') {
                            let part = part.trim();
                            if !part.is_empty() {
                                inners.push(part.to_string());
                            }
                        }
                    }
                    let mut result = vec![main];
                    result.append(&mut inners);
                    return result;
                }
                return vec![full_type];
            }
        }
        vec!["void".to_string()]
    }

    /// Helper to extract parameters and add them to a function.
    fn extract_parameters(&self, node: Node, source: &[u8], func: &mut Function) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if self
                .lang
                .parameter_container_patterns
                .iter()
                .any(|p| p == &child.kind())
            {
                let mut p_cursor = child.walk();
                for param in child.children(&mut p_cursor) {
                    if self
                        .lang
                        .parameter_patterns
                        .iter()
                        .any(|p| p == &param.kind())
                    {
                        let p_name = self.extract_identifier(param, source);
                        let types = self.extract_type(param, source);
                        let main_type =
                            types.first().cloned().unwrap_or_else(|| "void".to_string());
                        let inners = if types.len() > 1 {
                            types[1..].to_vec()
                        } else {
                            Vec::new()
                        };

                        if !p_name.is_empty() {
                            func.add_argument(Variable::new(p_name, main_type, inners));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    // helper function to set up parser for tests
    fn setup_parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        parser
    }

    #[test]
    fn test_variable_new() {
        let var = Variable::new("x".to_string(), "i32".to_string(), Vec::new());
        assert_eq!(var.name, "x");
        assert_eq!(var.var_type, "i32");
    }

    #[test]
    fn test_function_new() {
        let func = Function::new("test_func".to_string(), "void".to_string(), Vec::new());
        assert_eq!(func.name, "test_func");
        assert_eq!(func.return_type, "void");
        assert!(func.arguments.is_empty());
    }

    #[test]
    fn test_function_add_argument() {
        let mut func = Function::new("test_func".to_string(), "void".to_string(), Vec::new());
        let var = Variable::new("arg1".to_string(), "String".to_string(), Vec::new());
        func.add_argument(var);
        assert_eq!(func.arguments.len(), 1);
        assert_eq!(func.arguments[0].name, "arg1");
        assert_eq!(func.arguments[0].var_type, "String");
    }

    #[test]
    fn test_class_new() {
        let class = Class::new("MyClass".to_string());
        assert_eq!(class.name, "MyClass");
        assert_eq!(class.namespace, "");
        assert!(class.functions.is_empty());
        assert!(class.variables.is_empty());
    }

    #[test]
    fn test_class_with_namespace() {
        let class = Class::with_namespace("MyClass".to_string(), "my_namespace".to_string());
        assert_eq!(class.name, "MyClass");
        assert_eq!(class.namespace, "my_namespace");
    }

    #[test]
    fn test_class_add_items() {
        let mut class = Class::new("MyClass".to_string());
        let var = Variable::new("field1".to_string(), "u32".to_string(), Vec::new());
        let func = Function::new("method1".to_string(), "bool".to_string(), Vec::new());
        class.add_variable(var);
        class.add_function(func);
        assert_eq!(class.variables.len(), 1);
        assert_eq!(class.functions.len(), 1);
        assert_eq!(class.variables[0].name, "field1");
        assert_eq!(class.functions[0].name, "method1");
    }

    #[test]
    fn test_diagram_new() {
        let rust_config = LangConfig::load("rust");
        let diagram = Diagram::new(rust_config);
        assert!(diagram.classes.is_empty());
    }

    #[test]
    fn test_helpers_direct() {
        let mut parser = setup_parser();
        let source = b"fn test(val: i32) -> bool { true }";
        let rust_config = LangConfig::load("rust");

        let mut diagram = Diagram::new(rust_config);
        diagram.build(source, &mut parser);
        
        // Find the class or function we just built
        // In this case it's a top level function, so it might not be in a class
        // but navigate_node should have picked it up if function_patterns match.
        // Actually test_helpers_direct was testing extract_identifier directly.
        
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let func_node = root.child(0).unwrap();
        
        let name = diagram.extract_identifier(func_node, source);
        assert_eq!(name, "test");

        let types = diagram.extract_type(func_node, source);
        assert_eq!(types[0], "bool");
    }

    #[test]
    fn test_extract_type_generics() {
        let mut parser = setup_parser();
        let source = b"let x: Vec<User> = Vec::new();";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        // tree-sitter-rust: let_declaration -> variable_declaration -> ...
        // We'll just test the helper directly with mock data if needed,
        // but let's try to find the node.

        let rust_config = LangConfig::load("rust");
        let diagram = Diagram::new(rust_config);
        // find the type node

        fn find_type_node<'a>(node: Node<'a>, diagram: &Diagram) -> Option<Node<'a>> {
            if diagram.lang.type_patterns.iter().any(|p| p == node.kind())
                || node.kind().contains("type")
            {
                return Some(node);
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(n) = find_type_node(child, diagram) {
                    return Some(n);
                }
            }
            None
        }

        let type_node = find_type_node(root, &diagram);
        assert!(type_node.is_some());

        let types = diagram.extract_type(type_node.unwrap().parent().unwrap(), source);
        assert_eq!(types[0], "Vec");
        assert_eq!(types[1], "User");
    }
}
