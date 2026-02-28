use tree_sitter::Node;

pub struct Variable {
    pub name: String,
    pub var_type: String,
}

impl Variable {
    pub fn new(name: String, var_type: String) -> Self {
        Variable { name, var_type }
    }
}

pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub return_type: String,
}

impl Function {
    pub fn new(name: String, return_type: String) -> Self {
        Function {
            name,
            arguments: Vec::new(),
            return_type,
        }
    }

    pub fn add_argument(&mut self, arg: Variable) {
        self.arguments.push(arg);
    }
}

pub struct Class {
    pub name: String,
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Class {
            name,
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
}

// Constants that store the naming patterns for various languages
const FUNCTION_NODE_PATTERNS: [&str; 3] =
    ["function_item", "method_declaration", "function_definition"];
const VARIABLE_NODE_PATTERNS: [&str; 2] = ["field_declaration", "variable_declaration"];
const CLASS_NODE_PATTERNS: [&str; 3] = ["struct_item", "class_declaration", "class_specifier"];

impl Diagram {
    pub fn new() -> Self {
        Diagram {
            classes: Vec::new(),
        }
    }

    pub fn build(&mut self, root_node: Node, source: &[u8]) {
        self.navigate_node(root_node, source, None);
    }

    /// Recursively navigate the tree_sitter tree and build out Diagram
    pub fn navigate_node(&mut self, node: Node, source: &[u8], class_index: Option<usize>) {
        let kind = node.kind();
        let mut current_class_index = class_index;

        if CLASS_NODE_PATTERNS.contains(&kind) {
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                // update class index to match preexisting class if already exist
                if let Some(idx) = self.classes.iter().position(|class| class.name == name) {
                    current_class_index = Some(idx);
                } else {
                    // create new class and update indexes
                    self.classes.push(Class::new(name));
                    current_class_index = Some(self.classes.len() - 1);
                }
            }
        } else if kind == "impl_item" {
            // Rust specific: Implementation blocks
            let name = self.extract_text_by_kind(node, source, "type_identifier");
            if !name.is_empty() {
                if let Some(idx) = self.classes.iter().position(|class| class.name == name) {
                    current_class_index = Some(idx);
                } else {
                    self.classes.push(Class::new(name));
                    current_class_index = Some(self.classes.len() - 1);
                }
            }
        } else if FUNCTION_NODE_PATTERNS.contains(&kind) {
            // Function/Method detection
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                let mut func = Function::new(name, self.extract_type(node, source));
                self.extract_parameters(node, source, &mut func);

                if let Some(idx) = current_class_index {
                    self.classes[idx].add_function(func);
                }
            }
        } else if VARIABLE_NODE_PATTERNS.contains(&kind) {
            // Field/Variable detection
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                let var_type = self.extract_type(node, source);
                let var = Variable::new(name, var_type);
                if let Some(idx) = current_class_index {
                    self.classes[idx].add_variable(var);
                }
            }
        }

        // Recursively travel all children nodes (break case is handled by empty for loop)
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            self.navigate_node(child, source, current_class_index);
        }
    }

    /// Helper to extract text from a specific child kind.
    fn extract_text_by_kind(&self, node: Node, source: &[u8], kind: &str) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                    .to_string();
            }
        }
        String::new()
    }

    /// Helper to find identifiers (names) which may have different kind names across grammars.
    fn extract_identifier(&self, node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind == "identifier" || kind == "field_identifier" || kind == "type_identifier" {
                return String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                    .to_string();
            }
        }
        String::new()
    }

    /// Helper to extract type information from a node.
    fn extract_type(&self, node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            // Match nodes that typically represent types
            if kind.contains("type") || kind == "primitive_type" {
                return String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                    .to_string();
            }
        }
        "void".to_string()
    }

    /// Helper to extract parameters and add them to a function.
    fn extract_parameters(&self, node: Node, source: &[u8], func: &mut Function) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "parameters" || child.kind() == "formal_parameters" {
                let mut p_cursor = child.walk();
                for param in child.children(&mut p_cursor) {
                    if param.kind() == "parameter" || param.kind() == "formal_parameter" {
                        let p_name = self.extract_identifier(param, source);
                        let p_type = self.extract_type(param, source);
                        if !p_name.is_empty() {
                            func.add_argument(Variable::new(p_name, p_type));
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

    #[test]
    fn test_variable_new() {
        let var = Variable::new("x".to_string(), "i32".to_string());
        assert_eq!(var.name, "x");
        assert_eq!(var.var_type, "i32");
    }

    #[test]
    fn test_function_new() {
        let func = Function::new("test_func".to_string(), "void".to_string());
        assert_eq!(func.name, "test_func");
        assert_eq!(func.return_type, "void");
        assert!(func.arguments.is_empty());
    }

    #[test]
    fn test_function_add_argument() {
        let mut func = Function::new("test_func".to_string(), "void".to_string());
        let var = Variable::new("arg1".to_string(), "String".to_string());
        func.add_argument(var);
        assert_eq!(func.arguments.len(), 1);
        assert_eq!(func.arguments[0].name, "arg1");
        assert_eq!(func.arguments[0].var_type, "String");
    }

    #[test]
    fn test_class_new() {
        let class = Class::new("MyClass".to_string());
        assert_eq!(class.name, "MyClass");
        assert!(class.functions.is_empty());
        assert!(class.variables.is_empty());
    }

    #[test]
    fn test_class_add_items() {
        let mut class = Class::new("MyClass".to_string());
        let var = Variable::new("field1".to_string(), "u32".to_string());
        let func = Function::new("method1".to_string(), "bool".to_string());
        class.add_variable(var);
        class.add_function(func);
        assert_eq!(class.variables.len(), 1);
        assert_eq!(class.functions.len(), 1);
        assert_eq!(class.variables[0].name, "field1");
        assert_eq!(class.functions[0].name, "method1");
    }

    #[test]
    fn test_diagram_new() {
        let diagram = Diagram::new();
        assert!(diagram.classes.is_empty());
    }

    fn setup_parser() -> Parser {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        parser
    }

    #[test]
    fn test_diagram_build_simple_struct() {
        let mut parser = setup_parser();
        let source = std::fs::read("test_source_code_examples/simple_struct.rs")
            .expect("failed to read test file");
        let tree = parser.parse(&source, None).unwrap();
        let mut diagram = Diagram::new();
        diagram.build(tree.root_node(), &source);

        assert_eq!(diagram.classes.len(), 1);
        let class = &diagram.classes[0];
        assert_eq!(class.name, "User");
        assert_eq!(class.variables.len(), 3);
        assert_eq!(class.variables[0].name, "id");
        assert_eq!(class.variables[0].var_type, "u64");
        assert_eq!(class.variables[1].name, "username");
        assert_eq!(class.variables[1].var_type, "String");
        assert_eq!(class.variables[2].name, "email");
        assert_eq!(class.variables[2].var_type, "String");
    }

    #[test]
    fn test_diagram_build_impl_blocks() {
        let mut parser = setup_parser();
        let source = std::fs::read("test_source_code_examples/impl_blocks.rs")
            .expect("failed to read test file");
        let tree = parser.parse(&source, None).unwrap();
        let mut diagram = Diagram::new();
        diagram.build(tree.root_node(), &source);

        assert_eq!(diagram.classes.len(), 1);
        let class = &diagram.classes[0];
        assert_eq!(class.name, "Calculator");
        assert_eq!(class.functions.len(), 2);

        let add_func = &class.functions[0];
        assert_eq!(add_func.name, "add");
        assert_eq!(add_func.return_type, "i32");
        assert_eq!(add_func.arguments.len(), 2);
        assert_eq!(add_func.arguments[0].name, "a");
        assert_eq!(add_func.arguments[0].var_type, "i32");

        let clear_func = &class.functions[1];
        assert_eq!(clear_func.name, "clear");
    }

    #[test]
    fn test_diagram_build_complex_types() {
        let mut parser = setup_parser();
        let source = std::fs::read("test_source_code_examples/complex_types.rs")
            .expect("failed to read test file");
        let tree = parser.parse(&source, None).unwrap();
        let mut diagram = Diagram::new();
        diagram.build(tree.root_node(), &source);

        assert_eq!(diagram.classes.len(), 1);
        let class = &diagram.classes[0];
        assert_eq!(class.name, "ComplexData");
        assert_eq!(class.variables.len(), 1);
        assert_eq!(class.variables[0].name, "raw_bytes");
        // tree-sitter might return "Vec<u8>" or something similar
        assert!(class.variables[0].var_type.contains("Vec"));

        assert_eq!(class.functions.len(), 1);
        let func = &class.functions[0];
        assert_eq!(func.name, "process");
        assert_eq!(func.return_type, "bool");
        assert_eq!(func.arguments.len(), 1);
        assert_eq!(func.arguments[0].name, "mode");
        assert_eq!(func.arguments[0].var_type, "String");
    }

    #[test]
    fn test_diagram_build_generics() {
        let mut parser = setup_parser();
        let source = std::fs::read("test_source_code_examples/generics.rs")
            .expect("failed to read test file");
        let tree = parser.parse(&source, None).unwrap();
        let mut diagram = Diagram::new();
        diagram.build(tree.root_node(), &source);

        // For Box<T>, extract_text_by_kind for type_identifier might only return Box or it might fail if structure is different
        assert!(!diagram.classes.is_empty());
        let class = &diagram.classes[0];
        assert!(class.name.contains("Box"));

        assert_eq!(class.variables.len(), 1);
        assert_eq!(class.variables[0].name, "inner");

        // In impl<T> Box<T>, the type_identifier child of impl_item is "Box"
        // Let's see what it actually extracts.
    }

    #[test]
    fn test_helpers_direct() {
        let mut parser = setup_parser();
        let source = b"fn test(val: i32) -> bool { true }";
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let func_node = root.child(0).unwrap();

        let diagram = Diagram::new();
        let name = diagram.extract_identifier(func_node, source);
        assert_eq!(name, "test");

        let ret_type = diagram.extract_type(func_node, source);
        assert_eq!(ret_type, "bool");
    }
}
