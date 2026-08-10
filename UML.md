classDiagram
    class LangConfig {
        +file_extensions: Option~Vec<String>~
        +class_patterns: Option~Vec<String>~
        +function_patterns: Option~Vec<String>~
        +variable_patterns: Option~Vec<String>~
        +identifier_patterns: Option~Vec<String>~
        +type_patterns: Option~Vec<String>~
        +parameter_container_patterns: Option~Vec<String>~
        +parameter_patterns: Option~Vec<String>~
        +wrapper_patterns: Option~Vec<String>~
        +skip_patterns: Option~Vec<String>~
        +import_patterns: Option~Vec<String>~
        +namespace_patterns: Option~Vec<String>~
        +visibility_modifier_patterns: Option~Vec<String>~
        +private_by_default: Option~bool~
        +public_modifier_patterns: Option~Vec<String>~
        +private_modifier_patterns: Option~Vec<String>~
        +class_type_parameter_patterns: Option~Vec<String>~
        +type_field_names: Option~Vec<String>~
        +type_argument_patterns: Option~Vec<String>~
        +type_path_separator: Option~String~
        +type_strip_suffixes: Option~Vec<String>~
        +self_parameter_patterns: Option~Vec<String>~
        +type_annotation_strip_prefix: Option~String~
        +constructor_patterns: Option~Vec<String>~
        +infer_param_types_from_fields: Option~bool~
        +suppress_self_relations: Option~bool~
        +infer_unresolved_type_relations: Option~bool~
        +builtin_type_names: Option~Vec<String>~
        +loosely_typed: Option~bool~
        +all_configs() Vec~(String, Self)~
        +load(language: &str) Self
        +list_languages() Vec~&'static str~
    }
    class Variable {
        +var_type: String
        +inner_types: Option~Vec<String>~
        +name: Option~String~
        +private: bool
        +new(var_type: String) Self
        +void() Self
        +display_type() String
        +hidden_access_to_string() String
        +render(lang: &LangConfig) String
    }
    class Function {
        +name: String
        +arguments: Vec~Variable~
        +return_type: Variable
        +is_constructor: bool
        +new(name: String, return_type: Variable) Self
        +add_argument(arg: Variable) void
        +render(lang: &LangConfig) String
    }
    class Class {
        +name: String
        +namespace: String
        +type_params: Vec~String~
        +functions: Vec~Function~
        +variables: Vec~Variable~
        +new(name: String) Self
        +with_namespace(name: String, namespace: String) Self
        +display_name() String
        +add_function(func: Function) void
        +add_variable(var: Variable) void
    }
    class Diagram~'a~ {
        +classes: Vec~Class~
        +imports: Vec~String~
        -lang: &'a LangConfig
        +new(lang: &'a LangConfig) Self
        +lang() &LangConfig
        +build(source: &[u8], parser: Parser) void
        +navigate_node(node: Node, source: &[u8], class_index: Option~usize~, current_namespace: &str) void
        +try_import(node: Node, source: &[u8]) bool
        +try_namespace(node: Node, source: &[u8], active_namespace: &mut String) bool
        +try_class(node: Node, source: &[u8], active_namespace: &str, next_class_index: &mut Option~usize~) bool
        +try_function(node: Node, source: &[u8], next_class_index: Option~usize~) bool
        +try_variable(node: Node, source: &[u8], next_class_index: Option~usize~) bool
        +infer_param_types_from_fields(class_index: usize, func: &mut Function) void
        +extract_identifier(node: Node, source: &[u8]) String
        +extract_type(node: Node, source: &[u8]) Vec~String~
        +strip_type_suffixes(type_str: &str) String
        +strip_type_path(type_str: &str) String
        +extract_class_type_params(node: Node, source: &[u8]) Vec~String~
        +extract_visibility(node: Node, source: &[u8]) bool
        +extract_parameters(node: Node, source: &[u8], func: &mut Function) void
        +collect_parameters(container: Node, source: &[u8], func: &mut Function) void
    }
    class Edge {
        -source: String
        -destination: String
        -edge_type: Relation
    }
    class File~'a~ {
        +diagram: Diagram~'a~
    }
    class Directory~'a~ {
        +sub_directories: Vec~Directory<'a>~
        +files: Vec~File<'a>~
        +merged_diagram: Diagram~'a~
        +new(lang: &'a LangConfig) Self
        +merge_all() void
        +resolve_types(type_map: &GlobalTypeMap) void
    }
    class GlobalTypeMap {
        +types: HashMap~String, Vec<String>~
        +insert(short_name: String, qualified_name: String) void
        +resolve(type_name: &str, current_class_qualified: &str, _imports: &[String]) Option~String~
    }
    class Stitcher~'a~ {
        +root_path: PathBuf
        +type_map: GlobalTypeMap
        +config: LangConfig
        +parser: TreeSitterParser
        +no_ignore: bool
        +new(root_path: PathBuf, config: &'a LangConfig, parser: TreeSitterParser, no_ignore: bool) Self
        +build() Directory~'a~
        +collect_visible_paths() HashSet~PathBuf~
        +process_directory(current_path: &Path, current_dir: &mut Directory~'a~, visible: Option~&HashSet<PathBuf>~) void
        +is_source_file(path: &Path) bool
        +process_file(path: &Path) Option~File<'a>~
    }
    class Edge {
        -source: String
        -destination: String
        -edge_type: Relation
    }
    class Args {
        -list_languages: bool
        -lang: Option~String~
        -source_code: String
        -git: Option~String~
        -format: Format
        -code_block: bool
        -no_ignore: bool
        -destination: Option~String~
    }
    Function --> Variable
    Function --> Variable
    Class --> Function
    Class --> Variable
    Diagram --> Class
    File --> Diagram
    Directory --> Diagram
    Stitcher --> GlobalTypeMap
    Stitcher --> LangConfig
    Stitcher ..> Directory
