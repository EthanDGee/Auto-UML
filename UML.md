```mermaid
classDiagram
    class LangConfig {
        +class_patterns: Vec~String~
        +function_patterns: Vec~String~
        +variable_patterns: Vec~String~
        +identifier_patterns: Vec~String~
        +type_patterns: Vec~String~
        +parameter_container_patterns: Vec~String~
        +parameter_patterns: Vec~String~
        +wrapper_patterns: Vec~String~
        +skip_patterns: Vec~String~
        +import_patterns: Vec~String~
        +namespace_patterns: Vec~String~
        +load(language: &str) Self
    }
    class Edge {
        +source: String
        +destination: String
        +edge_type: Relation
    }
    class Variable {
        +name: String
        +var_type: String
        +inner_types: Vec~String~
        +new(name: String, var_type: String, inner_types: Vec~String~) Self
        +print_variable() void
    }
    class Function {
        +name: String
        +arguments: Vec~Variable~
        +return_type: String
        +return_inner_types: Vec~String~
        +new(name: String, return_type: String, return_inner_types: Vec~String~) Self
        +add_argument(arg: Variable) void
    }
    class Class {
        +name: String
        +namespace: String
        +functions: Vec~Function~
        +variables: Vec~Variable~
        +new(name: String) Self
        +with_namespace(name: String, namespace: String) Self
        +add_function(func: Function) void
        +add_variable(var: Variable) void
    }
    class Diagram {
        +classes: Vec~Class~
        +imports: Vec~String~
        +lang: LangConfig
        +new(language: &str) Self
        +build(root_node: Node, source: &[u8]) void
        +navigate_node(node: Node, source: &[u8], class_index: Option~usize~, current_namespace: &str) void
        +extract_identifier(node: Node, source: &[u8]) String
        +extract_type(node: Node, source: &[u8]) Vec~String~
        +extract_parameters(node: Node, source: &[u8], func: &mut Function) void
    }
    class File {
        +diagram: Diagram
    }
    class Directory {
        +sub_directories: Vec~Directory~
        +files: Vec~File~
        +merged_diagram: Diagram
        +new(lang: &str) Self
        +merge_all() void
        +resolve_types(type_map: &GlobalTypeMap) void
    }
    class GlobalTypeMap {
        +types: HashMap~String, Vec<String>~
        +new() Self
        +insert(short_name: String, qualified_name: String) void
        +resolve(type_name: &str, current_class_qualified: &str, _imports: &[String]) Option~String~
    }
    class Stitcher {
        +root_path: PathBuf
        +language: String
        +type_map: GlobalTypeMap
        +new(root_path: PathBuf, language: String) Self
        +build() Directory
        +process_directory(current_path: &Path, current_dir: &mut Directory) void
        +is_source_file(path: &Path) bool
        +process_file(path: &Path) Option~File~
    }
    class Args {
        +lang: Option~String~
        +source_code: Option~String~
        +git: Option~String~
        +no_mermaid: bool
        +destination: String
    }
    Function --> Variable
    Class --> Function
    Class --> Variable
    Diagram --> Class
    Diagram --> LangConfig
    File --> Diagram
    Directory --> Directory
    Directory --> File
    Directory --> Diagram
    Stitcher --> GlobalTypeMap
    Stitcher ..> Directory
    Stitcher ..> File

```