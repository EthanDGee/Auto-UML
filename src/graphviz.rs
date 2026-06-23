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

fn escape_label(s: &str) -> String {
    let mut escaped = String::new();
    let mut in_generic = false;
    for c in s.chars() {
        match c {
            '|' => escaped.push_str("\\|"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '<' => escaped.push_str("\\<"),
            '>' => escaped.push_str("\\>"),
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '~' => {
                if in_generic {
                    escaped.push_str("\\>");
                    in_generic = false;
                } else {
                    escaped.push_str("\\<");
                    in_generic = true;
                }
            }
            _ => escaped.push(c),
        }
    }
    escaped
}

pub fn generate(uml_diagram: &diagram::Diagram) -> String {
    let mut edges: Vec<Edge> = Vec::new();
    let mut output = String::from("digraph G {\n");
    output.push_str("    fontname=\"Helvetica,Arial,sans-serif\"\n");
    output.push_str("    node [fontname=\"Helvetica,Arial,sans-serif\", fontsize=10, shape=record, style=\"filled\", fillcolor=\"#f9f9f9\", color=\"#333333\"]\n");
    output.push_str(
        "    edge [fontname=\"Helvetica,Arial,sans-serif\", fontsize=10, color=\"#333333\"]\n",
    );
    output.push_str("    rankdir=BT\n\n");

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
            let sanitized_ns = namespace.replace(|c: char| !c.is_alphanumeric(), "_");
            output.push_str(&format!("    subgraph cluster_{} {{\n", sanitized_ns));
            output.push_str(&format!("        label = {:?};\n", namespace));
            output.push_str("        style = \"dashed\";\n");
            output.push_str("        color = \"#888888\";\n");
        }

        for class in classes {
            // Build the record label parts:
            // Group 1: Class Name
            let mut label_parts = vec![escape_label(&class.name)];

            // Group 2: Variables
            if !class.variables.is_empty() || !class.functions.is_empty() {
                let mut vars_str = String::new();
                for var in &class.variables {
                    vars_str.push_str(&format!("{}\\l", escape_label(&var.to_string())));
                }
                label_parts.push(vars_str);
            }

            // Group 3: Functions
            if !class.functions.is_empty() {
                let mut funcs_str = String::new();
                for func in &class.functions {
                    funcs_str.push_str(&format!("+{}\\l", escape_label(&func.to_string())));
                }
                label_parts.push(funcs_str);
            }

            let label = format!("{{{}}}", label_parts.join("|"));
            let indent = if has_namespace { "        " } else { "    " };
            output.push_str(&format!(
                "{}{:?} [label={:?}, shape=record];\n",
                indent, class.name, label
            ));

            // add edge if main type matches a qualified class name
            for var in &class.variables {
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

            // Add functions edges
            for func in &class.functions {
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
        }

        if has_namespace {
            output.push_str("    }\n");
        }
    }

    output.push_str("\n");

    // add edges to end of output
    for edge in edges {
        let edge_str = match edge.edge_type {
            Relation::Inheritance => format!(
                "    {:?} -> {:?} [arrowhead=\"empty\"];\n",
                edge.destination, edge.source
            ),
            Relation::Composition => format!(
                "    {:?} -> {:?} [arrowhead=\"diamond\"];\n",
                edge.destination, edge.source
            ),
            Relation::Aggregation => format!(
                "    {:?} -> {:?} [arrowhead=\"odiamond\"];\n",
                edge.destination, edge.source
            ),
            Relation::Realization => format!(
                "    {:?} -> {:?} [arrowhead=\"empty\", style=\"dashed\"];\n",
                edge.destination, edge.source
            ),
            Relation::Association => format!(
                "    {:?} -> {:?} [arrowhead=\"vee\"];\n",
                edge.source, edge.destination
            ),
            Relation::Dependency => format!(
                "    {:?} -> {:?} [arrowhead=\"vee\", style=\"dashed\"];\n",
                edge.source, edge.destination
            ),
            Relation::Link => format!(
                "    {:?} -> {:?} [arrowhead=\"none\"];\n",
                edge.source, edge.destination
            ),
        };
        output.push_str(&edge_str);
    }

    output.push_str("}\n");
    output
}

pub fn generate_code_block(uml_diagram: &diagram::Diagram) -> String {
    format!("```dot\n{}\n```", generate(uml_diagram))
}

#[cfg(test)]
mod tests {
    use crate::{
        diagram::{Class, Diagram, Function, Variable},
        graphviz::generate,
        lang_config::LangConfig,
    };

    #[test]
    fn test_graphviz_generation() {
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
        println!("Generated Graphviz output:\n{}", output);
        assert!(output.contains("digraph G"));
        assert!(output.contains("\"User\""));
        assert!(output.contains("label="));
        assert!(output.contains("id: u64"));
        assert!(output.contains("login(token:String) bool"));
    }
}
