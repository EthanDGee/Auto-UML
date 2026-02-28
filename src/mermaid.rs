use crate::diagram;

pub fn generate(uml_diagram: &diagram::Diagram) -> String {
    let mut output = String::from("classDiagram\n");

    for class in &uml_diagram.classes {
        output.push_str(&format!("    class {} {{\n", class.name));

        // Add variables
        for var in &class.variables {
            output.push_str(&format!("        +{}: {}\n", var.name, var.var_type));
        }

        // Add functions
        for func in &class.functions {
            let args: Vec<String> = func
                .arguments
                .iter()
                .map(|arg| format!("{}: {}", arg.name, arg.var_type))
                .collect();

            output.push_str(&format!(
                "        +{}({}) {}\n",
                func.name,
                args.join(", "),
                func.return_type
            ));
        }

        output.push_str("    }\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::{
        diagram::{Class, Diagram, Function, Variable},
        mermaid::generate,
    };

    #[test]
    fn test_mermaid_generation() {
        let mut diagram = Diagram::new("rust");
        let mut class = Class::new("User".to_string());

        class.add_variable(Variable::new("id".to_string(), "u64".to_string()));

        let mut func = Function::new("login".to_string(), "bool".to_string());
        func.add_argument(Variable::new("token".to_string(), "String".to_string()));
        class.add_function(func);

        diagram.classes.push(class);

        let output = generate(&diagram);
        assert!(output.contains("classDiagram"));
        assert!(output.contains("class User {"));
        assert!(output.contains("+id: u64"));
        assert!(output.contains("+login(token: String) bool"));
    }
}
