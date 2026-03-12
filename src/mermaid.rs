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

    // Group classes by namespace
    let mut namespace_map: std::collections::HashMap<String, Vec<&diagram::Class>> =
        std::collections::HashMap::new();
    for class in &uml_diagram.classes {
        namespace_map
            .entry(class.namespace.clone())
            .or_insert_with(Vec::new)
            .push(class);
    }

    for (namespace, classes) in namespace_map {
        let has_namespace = !namespace.is_empty();
        if has_namespace {
            output.push_str(&format!("    subgraph {}\n", namespace));
        }

        for class in classes {
            output.push_str(&format!("    class {} {{\n", class.name));

            // Add variables
            for var in &class.variables {
                let display_type = match &var.inner_types {
                    Some(inner) if !inner.is_empty() => {
                        format!("{}~{}~", var.var_type, inner.join(", "))
                    }
                    _ => var.var_type.clone(),
                };

                output.push_str(&format!("\t\t+{}\n", var.to_string()));

                // add edge if main type matches a qualified class name
                if let Some(destination) =
                    uml_diagram.classes.iter().find(|c| c.name == var.var_type)
                {
                    let new_edge = Edge {
                        source: class.name.clone(),
                        destination: destination.name.clone(),
                        edge_type: Relation::Association,
                    };
                    edges.push(new_edge);
                }
                // add edges for inner types
                for inner in &var.inner_types {
                    if let Some(destination) = uml_diagram
                        .classes
                        .iter()
                        .find(|c| c.name.as_ref().map(|n| n == inner).unwrap_or(false))
                    {
                        let new_edge = Edge {
                            source: class.name.clone(),
                            destination: destination.name.clone(),
                            edge_type: Relation::Association,
                        };
                        edges.push(new_edge);
                    }
                }
            }

            // Add functions
            for func in &class.functions {
                let args: Vec<String> = func.arguments.iter().map(|arg| arg.to_string()).collect();

                let display_return = if func.return_inner_types.is_empty() {
                    func.return_type.clone()
                } else {
                    format!(
                        "{}~{}~",
                        func.return_type,
                        func.return_inner_types.join(", ")
                    )
                };

                output.push_str(&format!(
                    "        +{}({}) {}\n",
                    func.name,
                    args.join(", "),
                    display_return
                ));

                // add edge if main return type matches a qualified class name
                if let Some(destination) = uml_diagram
                    .classes
                    .iter()
                    .find(|c| c.name == func.return_type)
                {
                    let new_edge = Edge {
                        source: class.name.clone(),
                        destination: destination.name.clone(),
                        edge_type: Relation::Dependency,
                    };
                    edges.push(new_edge);
                }
                // add edges for inner return types
                for inner in &func.return_inner_types {
                    if let Some(destination) = uml_diagram.classes.iter().find(|c| c.name == *inner)
                    {
                        let new_edge = Edge {
                            source: class.name.clone(),
                            destination: destination.name.clone(),
                            edge_type: Relation::Dependency,
                        };
                        edges.push(new_edge);
                    }
                }
            }

            output.push_str("    }\n");
        }

        if has_namespace {
            output.push_str("    end\n");
        }
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

pub fn generate_code_block(uml_diagram: &diagram::Diagram) -> String {
    format!("```mermaid\n{}\n```", generate(uml_diagram))
}

#[cfg(test)]
mod tests {
    use crate::{
        diagram::{Class, Diagram, Function, Variable},
        lang_config::LangConfig,
        mermaid::generate,
    };

    #[test]
    fn test_mermaid_generation() {
        let rust_config = LangConfig::load("rust");
        let mut diagram = Diagram::new(&rust_config);
        let mut class = Class::new("User".to_string());

        class.add_variable(Variable::named_variable(
            "id".to_string(),
            "u64".to_string(),
            None,
        ));

        let mut func = Function::new("login".to_string(), "bool".to_string(), Vec::new());
        func.add_argument(Variable::named_variable(
            "token".to_string(),
            "String".to_string(),
            None,
        ));
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
        let lang = LangConfig::load("rust");
        let mut diagram = Diagram::new(&lang);
        let mut user_class = Class::new("User".to_string());
        let session_class = Class::new("Session".to_string());
        let profile_class = Class::new("Profile".to_string());

        user_class.add_variable(Variable::named_variable(
            "current_session".to_string(),
            "Session".to_string(),
            None,
        ));
        user_class.add_function(Function::new(
            "get_profile".to_string(),
            "Profile".to_string(),
            Vec::new(),
        ));

        diagram.classes.push(user_class);
        diagram.classes.push(session_class);
        diagram.classes.push(profile_class);

        let output = generate(&diagram);
        assert!(output.contains("User --> Session"));
        assert!(output.contains("User ..> Profile"));
    }
}
