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

impl Diagram {
    pub fn new() -> Self {
        Diagram {
            classes: Vec::new(),
        }
    }

    /// Builds the diagram by traversing the AST.
    /// `source` is required to extract text (names/types) from node byte ranges.
    pub fn build(&mut self, _root_node: Node, _source: &[u8]) {
        // Implementation will follow based on the chosen language-agnostic strategy.
    }

    /// Recursively navigates the AST to identify UML-relevant structures.
    pub fn navigate_node(&mut self, _node: Node, _source: &[u8]) {
        // Implementation will follow based on the chosen language-agnostic strategy.
    }
}
