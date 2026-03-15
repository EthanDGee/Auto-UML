use crate::diagram;

const INDENT: &str = "    ";

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
            .or_default()
            .push(class);
    }

    for (namespace, classes) in namespace_map {
        let has_namespace = !namespace.is_empty();
        if has_namespace {
            output.push_str(&format!("namespace {} {{\n", namespace));
        }

        for class in classes {
            output.push_str(&format!("{}class {} {{\n", INDENT, class.name));

            // Add variables
            for var in &class.variables {
                output.push_str(&format!("{}{}{}\n", INDENT, INDENT, var));

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
                if let Some(inner_types) = &var.inner_types {
                    for inner in inner_types {
                        if let Some(destination) =
                            uml_diagram.classes.iter().find(|c| c.name == *inner)
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
            }

            // Add functions
            for func in &class.functions {
                output.push_str(&format!("{}{}+{}\n", INDENT, INDENT, func));

                // add edge if main return type matches a qualified class name
                if let Some(destination) = uml_diagram
                    .classes
                    .iter()
                    .find(|c| c.name == func.return_type.var_type)
                {
                    let new_edge = Edge {
                        source: class.name.clone(),
                        destination: destination.name.clone(),
                        edge_type: Relation::Dependency,
                    };
                    edges.push(new_edge);
                }
                // add edges for inner return types
                if let Some(inner_types) = &func.return_type.inner_types {
                    for inner in inner_types {
                        if let Some(destination) =
                            uml_diagram.classes.iter().find(|c| c.name == *inner)
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
            }

            output.push_str(&format!("{}}}\n", INDENT));
        }

        if has_namespace {
            output.push_str("}\n");
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
            "{}{} {} {}\n",
            INDENT, edge.source, arrow, edge.destination
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

        class.add_variable(Variable {
            name: Some("id".to_string()),
            var_type: "u64".to_string(),
            inner_types: None,
            private: false,
        });

        let mut func = Function::new("login".to_string(), Variable::new("bool".to_string()));
        func.add_argument(Variable {
            name: Some("token".to_string()),
            var_type: "String".to_string(),
            inner_types: None,
            private: false,
        });
        class.add_function(func);

        diagram.classes.push(class);

        let output = generate(&diagram);
        println!("Generated output:\n{}", output);
        assert!(output.contains("classDiagram"));
        assert!(output.contains("class User {"));
        assert!(output.contains("+id: u64"));
        assert!(output.contains("+login(token:String) bool"));
    }

    #[test]
    fn test_mermaid_edge_generation() {
        let lang = LangConfig::load("rust");
        let mut diagram = Diagram::new(&lang);
        let mut user_class = Class::new("User".to_string());
        let session_class = Class::new("Session".to_string());
        let profile_class = Class::new("Profile".to_string());

        user_class.add_variable(Variable {
            name: Some("current_session".to_string()),
            var_type: "Session".to_string(),
            inner_types: None,
            private: false,
        });
        user_class.add_function(Function::new(
            "get_profile".to_string(),
            Variable::new("Profile".to_string()),
        ));

        diagram.classes.push(user_class);
        diagram.classes.push(session_class);
        diagram.classes.push(profile_class);

        let output = generate(&diagram);
        assert!(output.contains("User --> Session"));
        assert!(output.contains("User ..> Profile"));
    }
}
