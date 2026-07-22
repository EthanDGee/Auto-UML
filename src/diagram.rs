use crate::lang_config::LangConfig;
use std::fmt;
use tree_sitter::Node;

const EMPTY_RETURN_TYPE: &str = "void";

pub struct Variable {
    pub var_type: String,
    pub inner_types: Option<Vec<String>>,
    pub name: Option<String>,
    pub private: bool,
}

impl Variable {
    pub fn new(var_type: String) -> Self {
        Variable {
            var_type,
            inner_types: None,
            name: None,
            private: false,
        }
    }

    pub fn void() -> Self {
        Variable::new(EMPTY_RETURN_TYPE.to_string())
    }

    pub fn display_type(&self) -> String {
        match &self.inner_types {
            Some(inner) if !inner.is_empty() => {
                format!("{}~{}~", self.var_type, inner.join(", "))
            }
            _ => self.var_type.clone(),
        }
    }

    /// A modification of the this functions fmt::Display where it doesn't show access modifiers,
    ///
    /// This is primarily used for things like function arguments
    pub fn hidden_access_to_string(&self) -> String {
        match &self.name {
            Some(name) if self.var_type != EMPTY_RETURN_TYPE => {
                format!("{}: {}", name, self.display_type())
            }
            Some(name) => name.clone(),
            None => self.display_type(),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let visibility = if self.private { "-" } else { "+" };
        match &self.name {
            Some(name) => write!(f, "{}{}: {}", visibility, name, self.display_type()),
            None => write!(f, "{}{}", visibility, self.display_type()),
        }
    }
}

pub struct Function {
    pub name: String,
    pub arguments: Vec<Variable>,
    pub return_type: Variable,
}

impl Function {
    pub fn new(name: String, return_type: Variable) -> Self {
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

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<String> = self
            .arguments
            .iter()
            .map(|arg| arg.hidden_access_to_string())
            .collect();
        write!(
            f,
            "{}({}) {}",
            self.name,
            args.join(", "),
            self.return_type.display_type()
        )
    }
}
pub struct Class {
    pub name: String,
    pub namespace: String,
    pub type_params: Vec<String>,
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
}

impl Class {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Class {
            name,
            namespace: String::new(),
            type_params: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn with_namespace(name: String, namespace: String) -> Self {
        Class {
            name,
            namespace,
            type_params: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn display_name(&self) -> String {
        if self.type_params.is_empty() {
            self.name.clone()
        } else {
            format!("{}~{}~", self.name, self.type_params.join(", "))
        }
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }

    pub fn add_variable(&mut self, var: Variable) {
        self.variables.push(var);
    }
}

/// Returns true if `kind` matches any pattern in `patterns`. A `None` or empty `patterns` never matches.
fn any_match(patterns: &Option<Vec<String>>, kind: &str) -> bool {
    patterns
        .as_ref()
        .is_some_and(|v| v.iter().any(|p| p == kind))
}

/// Like `any_match`, but preserves the `type_patterns` special case where the literal pattern
/// `"type"` also matches any kind containing the substring "type" (relied on by Rust's grammar).
fn any_type_match(patterns: &Option<Vec<String>>, kind: &str) -> bool {
    patterns.as_ref().is_some_and(|v| {
        v.iter()
            .any(|p| kind == p || (p == "type" && kind.contains("type")))
    })
}

pub struct Diagram<'a> {
    pub classes: Vec<Class>,
    pub imports: Vec<String>,
    lang: &'a LangConfig,
}

impl<'a> Diagram<'a> {
    pub fn new(lang: &'a LangConfig) -> Self {
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

    /// Recursively navigate the tree_sitter tree and build out Diagram.
    ///
    /// Each node is offered to the composite parts below in order; only the first part whose
    /// config flag is present AND matches the node's kind runs. A part with no configured
    /// pattern (`None`) never fires, so a language config only needs to declare the parts it
    /// actually uses.
    pub fn navigate_node(
        &mut self,
        node: Node,
        source: &[u8],
        class_index: Option<usize>,
        current_namespace: &str,
    ) {
        let mut next_class_index = class_index;
        let mut active_namespace = current_namespace.to_string();

        if self.try_import(node, source) {
        } else if self.try_namespace(node, source, &mut active_namespace) {
        } else if self.try_class(node, source, &active_namespace, &mut next_class_index) {
        } else if self.try_function(node, source, next_class_index) {
        } else if self.try_variable(node, source, next_class_index) {
        }

        // Recursively travel all children nodes (break case is handled by empty for loop)
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            self.navigate_node(child, source, next_class_index, &active_namespace);
        }
    }

    /// Gated on `import_patterns`. Returns whether the gate matched, not whether work was done.
    fn try_import(&mut self, node: Node, source: &[u8]) -> bool {
        let lang = self.lang;
        if !any_match(&lang.import_patterns, node.kind()) {
            return false;
        }
        let import_text =
            String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]).to_string();
        self.imports.push(import_text);
        true
    }

    /// Gated on `namespace_patterns`.
    fn try_namespace(&mut self, node: Node, source: &[u8], active_namespace: &mut String) -> bool {
        let lang = self.lang;
        if !any_match(&lang.namespace_patterns, node.kind()) {
            return false;
        }
        let name = self.extract_identifier(node, source);
        if !name.is_empty() {
            if active_namespace.is_empty() {
                *active_namespace = name;
            } else {
                *active_namespace = format!("{}_{}", active_namespace, name);
            }
        }
        true
    }

    /// Gated on `class_patterns`.
    fn try_class(
        &mut self,
        node: Node,
        source: &[u8],
        active_namespace: &str,
        next_class_index: &mut Option<usize>,
    ) -> bool {
        let lang = self.lang;
        if !any_match(&lang.class_patterns, node.kind()) {
            return false;
        }
        let name = self.extract_identifier(node, source);
        if !name.is_empty() {
            let type_params = self.extract_class_type_params(node, source);
            // update class index to match preexisting class if already exist
            if let Some(idx) = self
                .classes
                .iter()
                .position(|class| class.name == name && class.namespace == active_namespace)
            {
                if !type_params.is_empty() && self.classes[idx].type_params.is_empty() {
                    self.classes[idx].type_params = type_params;
                }
                *next_class_index = Some(idx);
            } else {
                // create new class and update indexes
                let mut new_class = Class::with_namespace(name, active_namespace.to_string());
                new_class.type_params = type_params;
                self.classes.push(new_class);
                *next_class_index = Some(self.classes.len() - 1);
            }
        }
        true
    }

    /// Gated on `function_patterns`.
    fn try_function(&mut self, node: Node, source: &[u8], next_class_index: Option<usize>) -> bool {
        let lang = self.lang;
        if !any_match(&lang.function_patterns, node.kind()) {
            return false;
        }
        let name = self.extract_identifier(node, source);
        if !name.is_empty() {
            let types = self.extract_type(node, source);
            let return_type = if types.is_empty() || types[0] == EMPTY_RETURN_TYPE {
                Variable::void()
            } else {
                let main_type = types[0].clone();
                let inners = if types.len() > 1 {
                    Some(types[1..].to_vec())
                } else {
                    None
                };
                Variable {
                    var_type: main_type,
                    inner_types: inners,
                    name: None,
                    private: false,
                }
            };
            let mut func = Function::new(name, return_type);
            self.extract_parameters(node, source, &mut func);

            if let Some(idx) = next_class_index {
                self.classes[idx].add_function(func);
            }
        }
        true
    }

    /// Gated on `variable_patterns`.
    fn try_variable(&mut self, node: Node, source: &[u8], next_class_index: Option<usize>) -> bool {
        let lang = self.lang;
        if !any_match(&lang.variable_patterns, node.kind()) {
            return false;
        }
        let name = self.extract_identifier(node, source);
        if !name.is_empty() {
            let types = self.extract_type(node, source);
            let main_type = types
                .first()
                .cloned()
                .unwrap_or_else(|| EMPTY_RETURN_TYPE.to_string());
            let inners = if types.len() > 1 {
                types[1..].to_vec()
            } else {
                Vec::new()
            };
            let is_private = self.extract_visibility(node, source);
            let var = Variable {
                var_type: main_type,
                name: Some(name),
                inner_types: Some(inners),
                private: is_private,
            };
            if let Some(idx) = next_class_index {
                self.classes[idx].add_variable(var);
            }
        }
        true
    }

    /// Helper to find identifiers (names) which may have different kind names across grammars.
    fn extract_identifier(&self, node: Node, source: &[u8]) -> String {
        let mut cursor = node.walk();
        let mut best_guess = String::new();

        for child in node.children(&mut cursor) {
            let kind = child.kind();

            if any_match(&self.lang.skip_patterns, kind) {
                continue;
            }
            if any_match(&self.lang.identifier_patterns, kind) {
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
            if any_match(&self.lang.wrapper_patterns, kind) {
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

            if any_match(&self.lang.skip_patterns, kind) {
                continue;
            }

            if any_type_match(&self.lang.type_patterns, kind) {
                let raw_text =
                    String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                        .to_string();
                let strip_prefix = self
                    .lang
                    .type_annotation_strip_prefix
                    .as_deref()
                    .filter(|s| !s.is_empty());
                let full_type = match strip_prefix {
                    Some(prefix) if raw_text.starts_with(prefix) => {
                        raw_text[prefix.len()..].trim().to_string()
                    }
                    _ => raw_text,
                };

                // Naive parsing of generics: "Vec<User>" -> ["Vec", "User"]
                if let Some(pos) = full_type.find('<') {
                    let raw_main = full_type[..pos].trim();
                    let main = self.strip_type_path(raw_main);
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
                return vec![self.strip_type_path(&full_type)];
            }

            // Recurse into certain nodes that wrap types
            if any_match(&self.lang.wrapper_patterns, kind) {
                let types = self.extract_type(child, source);
                if !types.is_empty() && types[0] != EMPTY_RETURN_TYPE {
                    return types;
                }
            }
        }
        vec![EMPTY_RETURN_TYPE.to_string()]
    }

    fn strip_type_path(&self, type_str: &str) -> String {
        let Some(sep) = self
            .lang
            .type_path_separator
            .as_deref()
            .filter(|s| !s.is_empty())
        else {
            return type_str.to_string();
        };
        type_str.rsplit(sep).next().unwrap_or(type_str).to_string()
    }

    /// Gated on `class_type_parameter_patterns` being present and non-empty.
    fn extract_class_type_params(&self, node: Node, source: &[u8]) -> Vec<String> {
        if !self
            .lang
            .class_type_parameter_patterns
            .as_ref()
            .is_some_and(|v| !v.is_empty())
        {
            return Vec::new();
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if any_match(&self.lang.class_type_parameter_patterns, child.kind()) {
                let text = String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                    .to_string();
                // strip surrounding < > and split by comma
                let inner = text.trim_start_matches('<').trim_end_matches('>');
                return inner
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    /// Helper to extract the visibility of variables and functions
    fn extract_visibility(&self, node: Node, source: &[u8]) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if any_match(&self.lang.visibility_modifier_patterns, kind) {
                let modifier =
                    String::from_utf8_lossy(&source[child.start_byte()..child.end_byte()])
                        .to_string();

                let is_private = self
                    .lang
                    .private_modifier_patterns
                    .as_ref()
                    .is_some_and(|v| v.contains(&modifier));
                let is_public = self
                    .lang
                    .public_modifier_patterns
                    .as_ref()
                    .is_some_and(|v| v.contains(&modifier));

                if is_private {
                    return true;
                } else if is_public {
                    return false;
                }
            }
        }
        self.lang.private_by_default.unwrap_or(false)
    }

    /// Helper to extract parameters and add them to a function.
    fn extract_parameters(&self, node: Node, source: &[u8], func: &mut Function) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if any_match(&self.lang.parameter_container_patterns, child.kind()) {
                let mut p_cursor = child.walk();
                for param in child.children(&mut p_cursor) {
                    let param_kind = param.kind();

                    // Handle bare consuming `self` (not &self or &mut self)
                    if any_match(&self.lang.self_parameter_patterns, param_kind) {
                        let text =
                            String::from_utf8_lossy(&source[param.start_byte()..param.end_byte()])
                                .to_string();
                        if !text.starts_with('&') {
                            func.add_argument(Variable {
                                var_type: EMPTY_RETURN_TYPE.to_string(),
                                name: Some("self".to_string()),
                                inner_types: None,
                                private: false,
                            });
                        }
                        continue;
                    }

                    if any_match(&self.lang.parameter_patterns, param_kind) {
                        let p_name = self.extract_identifier(param, source);
                        let types = self.extract_type(param, source);
                        let main_type = types
                            .first()
                            .cloned()
                            .unwrap_or_else(|| EMPTY_RETURN_TYPE.to_string());
                        let inners = if types.len() > 1 {
                            types[1..].to_vec()
                        } else {
                            Vec::new()
                        };

                        if !p_name.is_empty() {
                            func.add_argument(Variable {
                                var_type: main_type,
                                name: Some(p_name),
                                inner_types: Some(inners),
                                private: false,
                            });
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
        let var = Variable::new("i32".to_string());
        assert_eq!(var.var_type, "i32");
        assert_eq!(var.inner_types, None);
        assert_eq!(var.name, None);
    }

    #[test]
    fn test_function_new() {
        let func = Function::new("test_func".to_string(), Variable::void());
        assert_eq!(func.name, "test_func");
        assert_eq!(func.return_type.var_type, EMPTY_RETURN_TYPE);
        assert!(func.arguments.is_empty());
    }

    #[test]
    fn test_function_add_argument() {
        let mut func = Function::new("test_func".to_string(), Variable::void());
        let var = Variable {
            name: Some("arg1".to_string()),
            var_type: "String".to_string(),
            inner_types: None,
            private: false,
        };
        func.add_argument(var);
        assert_eq!(func.arguments.len(), 1);
        assert_eq!(func.arguments[0].var_type, "String");
        assert_eq!(func.arguments[0].name, Some("arg1".to_string()));
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
        let var = Variable {
            name: Some("field1".to_string()),
            var_type: "u32".to_string(),
            inner_types: None,
            private: false,
        };
        let func = Function::new("method1".to_string(), Variable::void());
        class.add_variable(var);
        class.add_function(func);
        assert_eq!(class.variables.len(), 1);
        assert_eq!(class.functions.len(), 1);
        assert_eq!(class.functions[0].name, "method1");
        assert_eq!(class.variables[0].name, Some("field1".to_string()));
    }

    #[test]
    fn test_diagram_new() {
        let rust_config = LangConfig::load("rust");
        let diagram = Diagram::new(&rust_config);
        assert!(diagram.classes.is_empty());
    }

    #[test]
    fn test_helpers_direct() {
        let mut parser = setup_parser();
        let source = b"fn test(val: i32) -> bool { true }";
        let rust_config = LangConfig::load("rust");

        let mut diagram = Diagram::new(&rust_config);
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
        let diagram = Diagram::new(&rust_config);
        // find the type node

        fn find_type_node<'a>(node: Node<'a>, diagram: &Diagram) -> Option<Node<'a>> {
            if diagram
                .lang
                .type_patterns
                .as_ref()
                .is_some_and(|v| v.iter().any(|p| p == node.kind()))
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
