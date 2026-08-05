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

fn relation_to_str(relation: Relation) -> &'static str {
    match relation {
        Relation::Inheritance => "<|--",
        Relation::Composition => "*--",
        Relation::Aggregation => "o--",
        Relation::Association => "-->",
        Relation::Link => "--",
        Relation::Dependency => "..>",
        Relation::Realization => "<|..",
    }
}

struct Edge {
    source: String,
    destination: String,
    edge_type: Relation,
}

pub fn generate(uml_diagram: &diagram::Diagram) -> String {
    let lang = uml_diagram.lang();
    let suppress_self_relations = lang.suppress_self_relations.unwrap_or(false);
    let infer_unresolved_type_relations = lang.infer_unresolved_type_relations.unwrap_or(false);
    let empty_builtin_types: Vec<String> = Vec::new();
    let builtin_type_names = lang
        .builtin_type_names
        .as_ref()
        .unwrap_or(&empty_builtin_types);
    let is_self_ref = |dest_name: &str, class: &diagram::Class| {
        suppress_self_relations && dest_name == class.name
    };

    let mut edges: Vec<Edge> = Vec::new();
    let mut output = String::from("classDiagram\n");

    for class in &uml_diagram.classes {
        output.push_str(&format!("{}class {} {{\n", INDENT, class.display_name()));

        // Add variables
        for var in &class.variables {
            output.push_str(&format!("{}{}{}\n", INDENT, INDENT, var.render(lang)));

            // add edge if main type matches a qualified class name (skip self-refs for generic classes)
            if let Some(destination) = uml_diagram.classes.iter().find(|c| {
                c.name == var.var_type
                    && !(c.name == class.name && !class.type_params.is_empty())
                    && !is_self_ref(&c.name, class)
            }) {
                let new_edge = Edge {
                    source: class.name.clone(),
                    destination: destination.name.clone(),
                    edge_type: Relation::Association,
                };
                edges.push(new_edge);
            } else if infer_unresolved_type_relations
                && var.inner_types.as_ref().is_none_or(|t| t.is_empty())
                && var.var_type.starts_with(|c: char| c.is_uppercase())
                && !is_self_ref(&var.var_type, class)
                && !builtin_type_names.contains(&var.var_type)
                && !class.type_params.contains(&var.var_type)
            {
                let new_edge = Edge {
                    source: class.name.clone(),
                    destination: var.var_type.clone(),
                    edge_type: Relation::Association,
                };
                edges.push(new_edge);
            }
            // add edges for inner types
            if let Some(inner_types) = &var.inner_types {
                for inner in inner_types {
                    if let Some(destination) = uml_diagram.classes.iter().find(|c| c.name == *inner)
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
            output.push_str(&format!("{}{}+{}\n", INDENT, INDENT, func.render(lang)));

            // add edge if main return type matches a qualified class name (skip self-refs for generic classes)
            if let Some(destination) = uml_diagram.classes.iter().find(|c| {
                c.name == func.return_type.var_type
                    && !(c.name == class.name && !class.type_params.is_empty())
                    && !is_self_ref(&c.name, class)
            }) {
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
        }

        output.push_str(&format!("{}}}\n", INDENT));
    }

    // add edges to end of output
    for edge in edges {
        let arrow = relation_to_str(edge.edge_type);
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
        assert!(output.contains("+login(token: String) bool"));
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
