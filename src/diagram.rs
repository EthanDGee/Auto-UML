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

        if CLASS_NODE_PATTERNS.iter().any(|&c| c == kind) {
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
        } else if kind == "impl_block" {
            // Rust specific: Implementation blocks as they require .extract_text_by_kind()
            let name = self.extract_text_by_kind(node, source, "type_identifier");
            if !name.is_empty() {
                if let Some(idx) = self.classes.iter().position(|class| class.name == name) {
                    current_class_index = Some(idx);
                } else {
                    self.classes.push(Class::new(name));
                    current_class_index = Some(self.classes.len() - 1);
                }
            }
        } else if FUNCTION_NODE_PATTERNS.iter().any(|&c| c == kind) {
            // Function/Method detection
            let name = self.extract_identifier(node, source);
            if !name.is_empty() {
                let mut func = Function::new(name, self.extract_type(node, source));
                self.extract_parameters(node, source, &mut func);

                if let Some(idx) = current_class_index {
                    self.classes[idx].add_function(func);
                }
            }
        } else if VARIABLE_NODE_PATTERNS.iter().any(|&c| c == kind) {
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
