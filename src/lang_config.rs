#[derive(Clone, Copy)]
pub struct LangConfig {
    pub class_patterns: &'static [&'static str],
    pub function_patterns: &'static [&'static str],
    pub variable_patterns: &'static [&'static str],
    pub identifier_patterns: &'static [&'static str],
    pub type_patterns: &'static [&'static str],
    pub parameter_container_patterns: &'static [&'static str],
    pub parameter_patterns: &'static [&'static str],
    pub wrapper_patterns: &'static [&'static str],
    pub skip_patterns: &'static [&'static str],
}

pub const RUST_CONFIG: LangConfig = LangConfig {
    class_patterns: &[
        "struct_item",
        "impl_item",
        "class_declaration",
        "class_specifier",
    ],
    function_patterns: &["function_item", "method_declaration", "function_definition"],
    variable_patterns: &["field_declaration", "variable_declaration"],
    identifier_patterns: &["identifier", "field_identifier", "type_identifier"],
    type_patterns: &["type", "primitive_type"],
    parameter_container_patterns: &["parameters", "formal_parameters"],
    parameter_patterns: &["parameter", "formal_parameter"],
    wrapper_patterns: &[
        "variable_declarator",
        "field_declaration",
        "function_item",
        "method_declaration",
        "class_declaration",
        "struct_item",
    ],
    skip_patterns: &[
        "modifiers",
        "visibility_modifier",
        "storage_class",
        "attribute_item",
        "type_parameters",
    ],
};

pub const JAVA_CONFIG: LangConfig = LangConfig {
    class_patterns: &[
        "class_declaration",
        "class_specifier",
        "interface_declaration",
    ],
    function_patterns: &[
        "method_declaration",
        "function_definition",
        "constructor_declaration",
    ],
    variable_patterns: &["field_declaration", "variable_declaration"],
    identifier_patterns: &["identifier", "field_identifier", "type_identifier"],
    type_patterns: &[
        "type",
        "primitive_type",
        "integral_type",
        "floating_point_type",
        "boolean_type",
    ],
    parameter_container_patterns: &["parameters", "formal_parameters"],
    parameter_patterns: &["parameter", "formal_parameter"],
    wrapper_patterns: &[
        "variable_declarator",
        "field_declaration",
        "function_item",
        "method_declaration",
        "class_declaration",
        "struct_item",
    ],
    skip_patterns: &[
        "modifiers",
        "visibility_modifier",
        "storage_class",
        "attribute_item",
        "type_parameters",
    ],
};
