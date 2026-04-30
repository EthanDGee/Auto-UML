use crate::diagram;
use regex::Regex;
use std::collections::{HashMap, HashSet};

const INDENT: &str = "    ";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Relation {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Link,
    Dependency,
    Realization,
}

impl std::str::FromStr for Relation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inheritance" => Ok(Relation::Inheritance),
            "composition" => Ok(Relation::Composition),
            "aggregation" => Ok(Relation::Aggregation),
            "association" => Ok(Relation::Association),
            "link" => Ok(Relation::Link),
            "dependency" => Ok(Relation::Dependency),
            "realization" => Ok(Relation::Realization),
            _ => Err(format!(
                "Unknown relation type: '{}'. Valid: inheritance, composition, aggregation, association, link, dependency, realization",
                s
            )),
        }
    }
}

pub struct FilterConfig {
    /// Regex patterns for input/start classes. If None, all classes are valid starts.
    pub filter_input: Option<Vec<String>>,
    /// Regex patterns for output/end classes. If None, all classes are valid ends.
    pub filter_output: Option<Vec<String>>,
    /// Which relation types to traverse and draw. If None, all types are used.
    pub allowed_relations: Option<HashSet<Relation>>,
}

fn relation_arrow(rel: &Relation) -> &'static str {
    match rel {
        Relation::Inheritance => "<|--",
        Relation::Composition => "*--",
        Relation::Aggregation => "o--",
        Relation::Association => "-->",
        Relation::Link => "--",
        Relation::Dependency => "..>",
        Relation::Realization => "<|..",
    }
}

fn is_relation_allowed(rel: &Relation, allowed: Option<&HashSet<Relation>>) -> bool {
    allowed.map_or(true, |set| set.contains(rel))
}

/// Build all edges from the diagram as (src_idx, dst_idx, Relation).
fn build_class_edges(diagram: &diagram::Diagram) -> Vec<(usize, usize, Relation)> {
    let mut edges = Vec::new();
    for (src_idx, class) in diagram.classes.iter().enumerate() {
        for var in &class.variables {
            if let Some(dest_idx) = diagram.classes.iter().position(|c| c.name == var.var_type) {
                edges.push((src_idx, dest_idx, Relation::Association));
            }
            if let Some(inner_types) = &var.inner_types {
                for inner in inner_types {
                    if let Some(dest_idx) = diagram.classes.iter().position(|c| c.name == *inner) {
                        edges.push((src_idx, dest_idx, Relation::Association));
                    }
                }
            }
        }
        for func in &class.functions {
            if let Some(dest_idx) = diagram
                .classes
                .iter()
                .position(|c| c.name == func.return_type.var_type)
            {
                edges.push((src_idx, dest_idx, Relation::Dependency));
            }
            if let Some(inner_types) = &func.return_type.inner_types {
                for inner in inner_types {
                    if let Some(dest_idx) = diagram.classes.iter().position(|c| c.name == *inner) {
                        edges.push((src_idx, dest_idx, Relation::Dependency));
                    }
                }
            }
        }
    }
    edges
}

fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
    patterns.iter().filter_map(|p| Regex::new(p).ok()).collect()
}

fn match_any_pattern(name: &str, regexes: &[Regex]) -> bool {
    regexes.iter().any(|re| re.is_match(name))
}

/// Iterative DFS returning all nodes reachable from `starts` in `adj`.
fn reachable_from(starts: &[usize], adj: &HashMap<usize, Vec<usize>>) -> HashSet<usize> {
    let mut visited = HashSet::new();
    let mut stack = starts.to_vec();
    while let Some(node) = stack.pop() {
        if visited.insert(node) {
            if let Some(neighbors) = adj.get(&node) {
                for &nb in neighbors {
                    if !visited.contains(&nb) {
                        stack.push(nb);
                    }
                }
            }
        }
    }
    visited
}

/// Returns the set of class indices to include given a filter.
///
/// A class is kept when it lies on at least one path from any start class to
/// any end class (both sets determined by the filter patterns). If one side is
/// unspecified, every class is considered a valid start or end for that side.
fn compute_kept_classes(
    diagram: &diagram::Diagram,
    filter: &FilterConfig,
    all_edges: &[(usize, usize, Relation)],
) -> HashSet<usize> {
    let allowed = filter.allowed_relations.as_ref();

    // Only traverse edges of allowed relation types.
    let mut forward_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut backward_adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(src, dst, ref rel) in all_edges {
        if is_relation_allowed(rel, allowed) {
            forward_adj.entry(src).or_default().push(dst);
            backward_adj.entry(dst).or_default().push(src);
        }
    }

    let all_indices: Vec<usize> = (0..diagram.classes.len()).collect();

    let forward_reachable = match &filter.filter_input {
        Some(patterns) => {
            let regexes = compile_patterns(patterns);
            let starts: Vec<usize> = diagram
                .classes
                .iter()
                .enumerate()
                .filter(|(_, c)| match_any_pattern(&c.name, &regexes))
                .map(|(i, _)| i)
                .collect();
            reachable_from(&starts, &forward_adj)
        }
        None => all_indices.iter().cloned().collect(),
    };

    let backward_reachable = match &filter.filter_output {
        Some(patterns) => {
            let regexes = compile_patterns(patterns);
            let targets: Vec<usize> = diagram
                .classes
                .iter()
                .enumerate()
                .filter(|(_, c)| match_any_pattern(&c.name, &regexes))
                .map(|(i, _)| i)
                .collect();
            reachable_from(&targets, &backward_adj)
        }
        None => all_indices.iter().cloned().collect(),
    };

    forward_reachable
        .intersection(&backward_reachable)
        .cloned()
        .collect()
}

pub fn generate(uml_diagram: &diagram::Diagram, filter: Option<&FilterConfig>) -> String {
    let all_edges = build_class_edges(uml_diagram);

    let allowed_relations = filter.and_then(|f| f.allowed_relations.as_ref());

    // Compute which classes to show (None means show all).
    let kept_classes: Option<HashSet<usize>> = filter.and_then(|f| {
        if f.filter_input.is_none() && f.filter_output.is_none() {
            None
        } else {
            Some(compute_kept_classes(uml_diagram, f, &all_edges))
        }
    });

    let mut output = String::from("classDiagram\n");

    // Group classes by namespace, skipping filtered-out classes.
    let mut namespace_map: std::collections::HashMap<String, Vec<&diagram::Class>> =
        std::collections::HashMap::new();
    for (idx, class) in uml_diagram.classes.iter().enumerate() {
        if kept_classes.as_ref().map_or(true, |k| k.contains(&idx)) {
            namespace_map
                .entry(class.namespace.clone())
                .or_default()
                .push(class);
        }
    }

    for (namespace, classes) in &namespace_map {
        let has_namespace = !namespace.is_empty();
        if has_namespace {
            output.push_str(&format!("namespace {} {{\n", namespace));
        }
        for class in classes {
            output.push_str(&format!("{}class {} {{\n", INDENT, class.name));
            for var in &class.variables {
                output.push_str(&format!("{}{}{}\n", INDENT, INDENT, var));
            }
            for func in &class.functions {
                output.push_str(&format!("{}{}+{}\n", INDENT, INDENT, func));
            }
            output.push_str(&format!("{}}}\n", INDENT));
        }
        if has_namespace {
            output.push_str("}\n");
        }
    }

    // Draw edges whose relation type is allowed and both endpoints are kept.
    for (src_idx, dst_idx, rel) in &all_edges {
        let rel_ok = is_relation_allowed(rel, allowed_relations);
        let class_ok = kept_classes
            .as_ref()
            .map_or(true, |k| k.contains(src_idx) && k.contains(dst_idx));
        if rel_ok && class_ok {
            output.push_str(&format!(
                "{}{} {} {}\n",
                INDENT,
                uml_diagram.classes[*src_idx].name,
                relation_arrow(rel),
                uml_diagram.classes[*dst_idx].name,
            ));
        }
    }

    output
}

pub fn generate_code_block(
    uml_diagram: &diagram::Diagram,
    filter: Option<&FilterConfig>,
) -> String {
    format!("```mermaid\n{}\n```", generate(uml_diagram, filter))
}

#[cfg(test)]
mod tests {
    use crate::{
        diagram::{Class, Diagram, Function, Variable},
        lang_config::LangConfig,
        mermaid::{FilterConfig, Relation, generate},
    };
    use std::collections::HashSet;

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

        let output = generate(&diagram, None);
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

        let output = generate(&diagram, None);
        assert!(output.contains("User --> Session"));
        assert!(output.contains("User ..> Profile"));
    }

    #[test]
    fn test_filter_input_only() {
        let lang = LangConfig::load("rust");
        let mut diagram = Diagram::new(&lang);

        let mut a = Class::new("A".to_string());
        let b = Class::new("B".to_string());
        let c = Class::new("C".to_string());
        // A -> B (association), no edge to C
        a.add_variable(Variable {
            name: Some("b".to_string()),
            var_type: "B".to_string(),
            inner_types: None,
            private: false,
        });
        diagram.classes.push(a);
        diagram.classes.push(b);
        diagram.classes.push(c);

        let filter = FilterConfig {
            filter_input: Some(vec!["^A$".to_string()]),
            filter_output: None,
            allowed_relations: None,
        };
        let output = generate(&diagram, Some(&filter));
        assert!(output.contains("class A {"));
        assert!(output.contains("class B {"));
        assert!(!output.contains("class C {"));
    }

    #[test]
    fn test_filter_input_and_output() {
        let lang = LangConfig::load("rust");
        let mut diagram = Diagram::new(&lang);

        let mut a = Class::new("A".to_string());
        let mut b = Class::new("B".to_string());
        let c = Class::new("C".to_string()); // unrelated
        // A -> B, B -> C but C is not a target, D is the target
        let d = Class::new("D".to_string());
        a.add_variable(Variable {
            name: Some("b".to_string()),
            var_type: "B".to_string(),
            inner_types: None,
            private: false,
        });
        b.add_variable(Variable {
            name: Some("d".to_string()),
            var_type: "D".to_string(),
            inner_types: None,
            private: false,
        });
        diagram.classes.push(a);
        diagram.classes.push(b);
        diagram.classes.push(c);
        diagram.classes.push(d);

        let filter = FilterConfig {
            filter_input: Some(vec!["^A$".to_string()]),
            filter_output: Some(vec!["^D$".to_string()]),
            allowed_relations: None,
        };
        let output = generate(&diagram, Some(&filter));
        assert!(output.contains("class A {"));
        assert!(output.contains("class B {"));
        assert!(!output.contains("class C {"));
        assert!(output.contains("class D {"));
    }

    #[test]
    fn test_filter_relations() {
        let lang = LangConfig::load("rust");
        let mut diagram = Diagram::new(&lang);

        let mut user = Class::new("User".to_string());
        let session = Class::new("Session".to_string());
        let profile = Class::new("Profile".to_string());

        user.add_variable(Variable {
            name: Some("session".to_string()),
            var_type: "Session".to_string(),
            inner_types: None,
            private: false,
        });
        user.add_function(Function::new(
            "get_profile".to_string(),
            Variable::new("Profile".to_string()),
        ));
        diagram.classes.push(user);
        diagram.classes.push(session);
        diagram.classes.push(profile);

        // Only allow Association edges — Dependency edges should be hidden
        let mut allowed = HashSet::new();
        allowed.insert(Relation::Association);
        let filter = FilterConfig {
            filter_input: None,
            filter_output: None,
            allowed_relations: Some(allowed),
        };
        let output = generate(&diagram, Some(&filter));
        assert!(output.contains("User --> Session"));
        assert!(!output.contains("User ..> Profile"));
    }
}
