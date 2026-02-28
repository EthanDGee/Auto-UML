pub struct Variable {
    pub name: String,
    pub var_type: String,
}

pub struct Function {
    pub name: String,
    arguments: Vec<Variable>,
    return_type: String,
}

pub struct Class {
    pub name: String,
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
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
}
