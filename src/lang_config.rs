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
    pub import_patterns: &'static [&'static str],
    pub namespace_patterns: &'static [&'static str],
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
    import_patterns: &["use_declaration"],
    namespace_patterns: &["mod_item"],
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
    import_patterns: &["import_declaration"],
    namespace_patterns: &["package_declaration"],
};

pub const JAVASCRIPT_CONFIG: LangConfig = LangConfig {
    class_patterns: &["class_declaration"],
    function_patterns: &["method_definition", "function_declaration"],
    variable_patterns: &[
        "public_field_definition",
        "field_definition",
        "variable_declarator",
    ],
    identifier_patterns: &["identifier", "property_identifier", "type_identifier"],
    type_patterns: &[],
    parameter_container_patterns: &["formal_parameters"],
    parameter_patterns: &["identifier"],
    wrapper_patterns: &["lexical_declaration", "variable_declaration"],
    skip_patterns: &[],
    import_patterns: &["import_statement"],
    namespace_patterns: &[],
};

pub const CSHARP_CONFIG: LangConfig = LangConfig {
    class_patterns: &[
        "class_declaration",
        "interface_declaration",
        "struct_declaration",
    ],
    function_patterns: &["method_declaration", "constructor_declaration"],
    variable_patterns: &["field_declaration", "variable_declarator"],
    identifier_patterns: &["identifier", "variable_identifier", "type_identifier"],
    type_patterns: &["type", "predefined_type"],
    parameter_container_patterns: &["parameter_list"],
    parameter_patterns: &["parameter"],
    wrapper_patterns: &["variable_declaration"],
    skip_patterns: &["modifier"],
    import_patterns: &["using_directive"],
    namespace_patterns: &["namespace_declaration"],
};

pub const CPP_CONFIG: LangConfig = LangConfig {
    class_patterns: &["class_specifier", "struct_specifier"],
    function_patterns: &["function_definition", "declaration"],
    variable_patterns: &["field_declaration", "declaration"],
    identifier_patterns: &["identifier", "field_identifier", "type_identifier"],
    type_patterns: &["primitive_type", "type_identifier"],
    parameter_container_patterns: &["parameter_list"],
    parameter_patterns: &["parameter_declaration"],
    wrapper_patterns: &[
        "field_declaration",
        "function_definition",
        "function_declarator",
    ],
    skip_patterns: &["storage_class_specifier", "type_qualifier"],
    import_patterns: &["preproc_include"],
    namespace_patterns: &["namespace_definition"],
};

pub const TYPESCRIPT_CONFIG: LangConfig = LangConfig {
    class_patterns: &["class_declaration", "interface_declaration"],
    function_patterns: &["method_definition", "function_declaration"],
    variable_patterns: &[
        "public_field_definition",
        "field_definition",
        "variable_declarator",
    ],
    identifier_patterns: &["identifier", "property_identifier", "type_identifier"],
    type_patterns: &["type_annotation", "primitive_type"],
    parameter_container_patterns: &["formal_parameters"],
    parameter_patterns: &["required_parameter", "optional_parameter"],
    wrapper_patterns: &["lexical_declaration", "variable_declaration"],
    skip_patterns: &[],
    import_patterns: &["import_statement"],
    namespace_patterns: &[],
};

pub const OBJC_CONFIG: LangConfig = LangConfig {
    class_patterns: &[
        "class_interface",
        "category_interface",
        "protocol_declaration",
    ],
    function_patterns: &["method_definition", "method_declaration"],
    variable_patterns: &["property_declaration", "field_declaration"],
    identifier_patterns: &["identifier", "property_name", "type_identifier"],
    type_patterns: &[
        "type_name",
        "primitive_type",
        "type_identifier",
        "sized_type_specifier",
    ],
    parameter_container_patterns: &["parameter_list"],
    parameter_patterns: &["parameter_declaration"],
    wrapper_patterns: &[
        "property_declaration",
        "translation_unit",
        "struct_declaration",
        "struct_declarator",
        "pointer_declarator",
    ],
    skip_patterns: &[
        "property_attributes",
        "@interface",
        "@implementation",
        "@property",
        "@end",
        "NSObject",
        ":",
        ";",
        "(",
        ")",
        "*",
        "sized_type_specifier",
        "typedefed_specifier",
        "long",
        "class_implementation",
        "implementation_definition",
    ],
    import_patterns: &["preproc_import", "preproc_include"],
    namespace_patterns: &[],
};

pub const DART_CONFIG: LangConfig = LangConfig {
    class_patterns: &["class_definition", "enum_definition", "mixin_definition"],
    function_patterns: &[
        "method_declaration",
        "constructor_signature",
        "function_signature",
    ],
    variable_patterns: &[
        "field_declaration",
        "variable_declaration",
        "initialized_identifier",
    ],
    identifier_patterns: &["identifier", "type_identifier"],
    type_patterns: &["type_annotation", "type_identifier", "void_type"],
    parameter_container_patterns: &["formal_parameter_list"],
    parameter_patterns: &[
        "normal_formal_parameter",
        "field_formal_parameter",
        "formal_parameter",
    ],
    wrapper_patterns: &[
        "declaration",
        "variable_declaration_list",
        "class_member_definition",
        "method_signature",
        "initialized_identifier_list",
        "program",
    ],
    skip_patterns: &[
        "final", "const", "static", "var", "late", "class", "void", "this", "{", "}", "(", ")", ";",
    ],
    import_patterns: &["import_directive"],
    namespace_patterns: &["library_directive"],
};
