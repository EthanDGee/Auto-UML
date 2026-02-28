use crate::diagram;

#[allow(dead_code)]
enum Relation {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Link,
    Dependency,
    Realization,
}

struct Edge {
    source: String,
    destination: String,
    edge_type: Relation,
}

pub fn generate(uml_diagram: &diagram::Diagram) -> String {
    let mut edges: Vec<Edge> = Vec::new();

    let mut output = String::from("classDiagram\n");

    for class in &uml_diagram.classes {
        output.push_str(&format!("    class {} {{\n", class.name));

        // Add variables
        for var in &class.variables {
            output.push_str(&format!("        +{}: {}\n", var.name, var.var_type));

            // add edge if variable matches class
            if let Some(destination) = uml_diagram.classes.iter().find(|c| c.name == var.var_type) {
                let new_edge = Edge {
                    source: class.name.clone(),
                    destination: destination.name.clone(),
                    edge_type: Relation::Association,
                };
                edges.push(new_edge);
            }
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

            // add edge if return type matches class
            if let Some(destination) = uml_diagram.classes.iter().find(|c| c.name == func.return_type) {
                let new_edge = Edge {
                    source: class.name.clone(),
                    destination: destination.name.clone(),
                    edge_type: Relation::Dependency,
                };
                edges.push(new_edge);
            }
        }

        output.push_str("    }\n");
    }

    // add edges to end of output
    for edge in edges {
        let arrow = match edge.edge_type {
            Relation::Inheritance => "<|--",
            Relation::Composition => "*--",
            Relation::Aggregation => "o--",
            Relation::Association => "-->",
            Relation::Link => "--",
            Relation::Dependency => "..>",
            Relation::Realization => "<|..",
        };
        output.push_str(&format!(
            "    {} {} {}\n",
            edge.source, arrow, edge.destination
        ));
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

    #[test]
    fn test_mermaid_edge_generation() {
        let mut diagram = Diagram::new("rust");
        let mut user_class = Class::new("User".to_string());
        let session_class = Class::new("Session".to_string());
        let profile_class = Class::new("Profile".to_string());

        user_class.add_variable(Variable::new("current_session".to_string(), "Session".to_string()));
        user_class.add_function(Function::new("get_profile".to_string(), "Profile".to_string()));

        diagram.classes.push(user_class);
        diagram.classes.push(session_class);
        diagram.classes.push(profile_class);

        let output = generate(&diagram);
        assert!(output.contains("User --> Session"));
        assert!(output.contains("User ..> Profile"));
    }
}
