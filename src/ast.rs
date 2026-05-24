use crate::error::Result;
use crate::layout::relative_display;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const SKIP_NAMESPACES: &[&str] = &["std", "__gnu_cxx", "__cxx11", "__detail", "__1"];
const SYSTEM_PREFIXES: &[&str] = &[
    "/usr/include",
    "/usr/lib",
    "/usr/local/include",
    "/Applications/Xcode",
    "/Library/Developer",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TranslationUnit {
    pub source_path: PathBuf,
    pub module_name: String,
    pub functions: Vec<FunctionDecl>,
    pub classes: Vec<ClassDecl>,
    pub enums: Vec<EnumDecl>,
    pub type_aliases: Vec<TypeAliasDecl>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FunctionDecl {
    pub name: String,
    pub qualified_name: String,
    pub return_type: String,
    pub params: Vec<ParameterDecl>,
    pub is_inline: bool,
    pub is_static: bool,
    pub is_template: bool,
    pub throws: bool,
    pub source_range: Option<SourceRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ParameterDecl {
    pub name: String,
    pub qual_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClassDecl {
    pub name: String,
    pub qualified_name: String,
    pub tag: String,
    pub constructors: Vec<ConstructorDecl>,
    pub destructor: Option<MethodDecl>,
    pub methods: Vec<MethodDecl>,
    pub bases: Vec<String>,
    pub is_abstract: bool,
    pub is_template: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ConstructorDecl {
    pub params: Vec<ParameterDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MethodDecl {
    pub name: String,
    pub qualified_name: String,
    pub return_type: String,
    pub params: Vec<ParameterDecl>,
    pub is_const: bool,
    pub is_volatile: bool,
    pub ref_qualifier: Option<String>,
    pub is_pure: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub throws: bool,
    #[serde(default)]
    pub is_template: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumDecl {
    pub name: String,
    pub qualified_name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TypeAliasDecl {
    pub name: String,
    pub qualified_name: String,
    pub underlying_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceRange {
    pub begin: usize,
    pub end: usize,
}

pub fn parse_translation_unit(ast_json_path: &Path, source_path: &Path) -> Result<TranslationUnit> {
    let json = fs::read_to_string(ast_json_path)?;
    let root: Value = serde_json::from_str(&json)?;
    let source_text = fs::read_to_string(source_path).unwrap_or_default();
    let module_name = module_name_from_source(source_path);
    let matcher = FileMatcher::new(source_path)?;
    let mut parser = Parser {
        matcher,
        source_text,
        tu: TranslationUnit {
            source_path: source_path.to_path_buf(),
            module_name,
            ..TranslationUnit::default()
        },
        seen_functions: IndexSet::new(),
        seen_classes: IndexSet::new(),
        seen_enums: IndexSet::new(),
        seen_aliases: IndexSet::new(),
    };

    for child in children(&root) {
        parser.visit_top_level(child, &[]);
    }
    Ok(parser.tu)
}

pub fn module_name_from_source(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module");
    let mut out = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "module".into()
    } else {
        out
    }
}

struct Parser {
    matcher: FileMatcher,
    source_text: String,
    tu: TranslationUnit,
    seen_functions: IndexSet<String>,
    seen_classes: IndexSet<String>,
    seen_enums: IndexSet<String>,
    seen_aliases: IndexSet<String>,
}

impl Parser {
    fn visit_top_level(&mut self, node: &Value, namespaces: &[String]) {
        match kind(node) {
            Some("NamespaceDecl") => {
                if let Some(name) = name(node) {
                    if SKIP_NAMESPACES.contains(&name) || name.starts_with("__") {
                        return;
                    }
                    let mut next = namespaces.to_vec();
                    next.push(name.to_string());
                    for child in children(node) {
                        self.visit_top_level(child, &next);
                    }
                }
            }
            Some("FunctionDecl") => {
                if let Some(function) = self.parse_function(node, namespaces, false) {
                    let key = signature_key(&function.qualified_name, &function.params);
                    if self.seen_functions.insert(key) {
                        self.tu.functions.push(function);
                    }
                }
            }
            Some("FunctionTemplateDecl") => {
                for child in children(node) {
                    if kind(child) == Some("FunctionDecl") {
                        if let Some(mut function) = self.parse_function(child, namespaces, false) {
                            function.is_template = true;
                            let key = signature_key(&function.qualified_name, &function.params);
                            if self.seen_functions.insert(key) {
                                self.tu.functions.push(function);
                            }
                        }
                    }
                }
            }
            Some("CXXRecordDecl") | Some("ClassTemplateSpecializationDecl") => {
                if let Some(class) = self.parse_class(node, namespaces) {
                    if self.seen_classes.insert(class.qualified_name.clone()) {
                        self.tu.classes.push(class);
                    }
                }
            }
            Some("ClassTemplateDecl") => {
                for child in children(node) {
                    if matches!(kind(child), Some("CXXRecordDecl") | Some("ClassTemplateSpecializationDecl")) {
                        if let Some(mut class) = self.parse_class(child, namespaces) {
                            class.is_template = true;
                            if self.seen_classes.insert(class.qualified_name.clone()) {
                                self.tu.classes.push(class);
                            }
                        }
                    }
                }
            }
            Some("EnumDecl") => {
                if let Some(enm) = self.parse_enum(node, namespaces) {
                    if self.seen_enums.insert(enm.qualified_name.clone()) {
                        self.tu.enums.push(enm);
                    }
                }
            }
            Some("TypedefDecl") | Some("TypeAliasDecl") => {
                if let Some(alias) = self.parse_alias(node, namespaces) {
                    if self.seen_aliases.insert(alias.qualified_name.clone()) {
                        self.tu.type_aliases.push(alias);
                    }
                }
            }
            Some("LinkageSpecDecl") | Some("ExternCContextDecl") => {
                for child in children(node) {
                    self.visit_top_level(child, namespaces);
                }
            }
            _ => {}
        }
    }

    fn parse_function(
        &self,
        node: &Value,
        namespaces: &[String],
        from_friend: bool,
    ) -> Option<FunctionDecl> {
        let loc = node_location(node)?;
        if !self.matcher.matches(&loc) {
            return None;
        }
        let raw_name = name(node)?;
        if raw_name.starts_with("__") {
            return None;
        }
        if !from_friend && is_methodish_name(raw_name) {
            return None;
        }
        let qualified_name = qualified_name(namespaces, raw_name);
        let params = children(node)
            .iter()
            .filter(|child| kind(child) == Some("ParmVarDecl"))
            .map(parse_param)
            .collect::<Vec<_>>();
        let return_type = node
            .get("type")
            .and_then(|v| v.get("qualType"))
            .and_then(Value::as_str)
            .map(function_return_from_qualtype)
            .unwrap_or_else(|| "void".into());

        Some(FunctionDecl {
            name: raw_name.into(),
            qualified_name,
            return_type,
            params,
            is_inline: node.get("inline").and_then(Value::as_bool).unwrap_or(false)
                || source_slice(&self.source_text, range(node)).contains("inline "),
            is_static: node.get("storageClass").and_then(Value::as_str) == Some("static"),
            is_template: false,
            throws: source_slice(&self.source_text, range(node)).contains("throw"),
            source_range: range(node),
        })
    }

    fn parse_class(&mut self, node: &Value, namespaces: &[String]) -> Option<ClassDecl> {
        let loc = node_location(node)?;
        if !self.matcher.matches(&loc) {
            return None;
        }
        if !node
            .get("completeDefinition")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return None;
        }
        let raw_name = name(node)?;
        if raw_name.is_empty() || raw_name.starts_with("__") {
            return None;
        }
        let qualified_name = qualified_name(namespaces, raw_name);
        let mut class = ClassDecl {
            name: raw_name.into(),
            qualified_name,
            tag: node.get("tagUsed").and_then(Value::as_str).unwrap_or("class").into(),
            bases: parse_bases(node),
            is_template: false,
            ..ClassDecl::default()
        };
        for child in children(node) {
            match kind(child) {
                Some("CXXConstructorDecl") => {
                    if self.matcher.matches(&node_location(child).unwrap_or_default()) {
                        class.constructors.push(ConstructorDecl {
                            params: children(child)
                                .iter()
                                .filter(|grand| kind(grand) == Some("ParmVarDecl"))
                                .map(parse_param)
                                .collect(),
                        });
                    }
                }
                Some("CXXDestructorDecl") => {
                    class.destructor = self.parse_method(child, namespaces, raw_name);
                }
                Some("CXXMethodDecl") => {
                    if let Some(method) = self.parse_method(child, namespaces, raw_name) {
                        if method.is_pure {
                            class.is_abstract = true;
                        }
                        class.methods.push(method);
                    }
                }
                Some("FunctionTemplateDecl") => {
                    for grand in children(child) {
                        if kind(grand) == Some("CXXMethodDecl") {
                            if let Some(mut method) = self.parse_method(grand, namespaces, raw_name) {
                                method.is_template = true;
                                class.methods.push(method);
                            }
                        }
                    }
                }
                Some("CXXRecordDecl") | Some("ClassTemplateSpecializationDecl") => {
                    // Parse nested class definitions and add them to the translation unit.
                    let mut nested_ns = namespaces.to_vec();
                    nested_ns.push(raw_name.to_string());
                    if let Some(nested) = self.parse_class(child, &nested_ns) {
                        if self.seen_classes.insert(nested.qualified_name.clone()) {
                            self.tu.classes.push(nested);
                        }
                    }
                }
                Some("FriendDecl") => {
                    for grand in children(child) {
                        if kind(grand) == Some("FunctionDecl") {
                            if let Some(function) = self.parse_function(grand, namespaces, true) {
                                let key = signature_key(&function.qualified_name, &function.params);
                                if self.seen_functions.insert(key) {
                                    self.tu.functions.push(function);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Some(class)
    }

    fn parse_method(&self, node: &Value, namespaces: &[String], class_name: &str) -> Option<MethodDecl> {
        let loc = node_location(node)?;
        if !self.matcher.matches(&loc) {
            return None;
        }
        let raw_name = name(node)?;
        if raw_name.starts_with("__") {
            return None;
        }
        let qual_type = node
            .get("type")
            .and_then(|v| v.get("qualType"))
            .and_then(Value::as_str)
            .unwrap_or("void()");
        let params = children(node)
            .iter()
            .filter(|child| kind(child) == Some("ParmVarDecl"))
            .map(parse_param)
            .collect::<Vec<_>>();
        Some(MethodDecl {
            name: raw_name.into(),
            qualified_name: format!("{}::{}", qualified_name(namespaces, class_name), raw_name),
            return_type: function_return_from_qualtype(qual_type),
            params,
            is_const: qual_type.contains(" const"),
            is_volatile: qual_type.contains(" volatile"),
            ref_qualifier: if qual_type.contains(" &&") {
                Some("&&".into())
            } else if qual_type.contains(" &") {
                Some("&".into())
            } else {
                None
            },
            is_pure: node.get("pure").and_then(Value::as_bool).unwrap_or(false)
                || node.get("isPure").and_then(Value::as_bool).unwrap_or(false),
            is_static: node.get("storageClass").and_then(Value::as_str) == Some("static"),
            is_virtual: node.get("virtual").and_then(Value::as_bool).unwrap_or(false)
                || qual_type.contains("virtual"),
            throws: source_slice(&self.source_text, range(node)).contains("throw"),
            is_template: false,
        })
    }

    fn parse_enum(&self, node: &Value, namespaces: &[String]) -> Option<EnumDecl> {
        let loc = node_location(node)?;
        if !self.matcher.matches(&loc) {
            return None;
        }
        let raw_name = name(node)?;
        if raw_name.is_empty() || raw_name.starts_with("__") {
            return None;
        }
        let variants = children(node)
            .iter()
            .filter(|child| kind(child) == Some("EnumConstantDecl"))
            .map(|child| EnumVariant {
                name: name(child).unwrap_or_default().into(),
                value: child
                    .get("inner")
                    .and_then(Value::as_array)
                    .and_then(|inner| inner.last())
                    .and_then(|leaf| leaf.get("value"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            })
            .collect();
        Some(EnumDecl {
            name: raw_name.into(),
            qualified_name: qualified_name(namespaces, raw_name),
            variants,
        })
    }

    fn parse_alias(&self, node: &Value, namespaces: &[String]) -> Option<TypeAliasDecl> {
        let loc = node_location(node)?;
        if !self.matcher.matches(&loc) {
            return None;
        }
        let raw_name = name(node)?;
        if raw_name.starts_with("__") {
            return None;
        }
        let underlying_type = node
            .get("type")
            .and_then(|v| v.get("qualType"))
            .and_then(Value::as_str)?;
        Some(TypeAliasDecl {
            name: raw_name.into(),
            qualified_name: qualified_name(namespaces, raw_name),
            underlying_type: underlying_type.into(),
        })
    }
}

#[derive(Default)]
struct FileMatcher {
    accepted: IndexSet<String>,
}

impl FileMatcher {
    fn new(source_path: &Path) -> Result<Self> {
        let mut accepted = IndexSet::new();
        let canonical = fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf());
        accepted.insert(normalize_path(&canonical));
        accepted.insert(normalize_path(source_path));
        if let Some(name) = source_path.file_name().and_then(|s| s.to_str()) {
            accepted.insert(name.to_string());
        }
        Ok(Self { accepted })
    }

    fn matches(&self, loc: &str) -> bool {
        // Nodes whose location was inherited from the main TU file always match.
        if loc == MAIN_TU_FILE {
            return true;
        }
        if is_system_path(loc) {
            return false;
        }
        let normalized = normalize_loc(loc);
        self.accepted.contains(&normalized)
            || self
                .accepted
                .iter()
                .any(|candidate| normalized.ends_with(candidate) || candidate.ends_with(&normalized))
    }
}

fn parse_param(node: &Value) -> ParameterDecl {
    ParameterDecl {
        name: name(node).unwrap_or("arg").into(),
        qual_type: node
            .get("type")
            .and_then(|v| v.get("qualType"))
            .and_then(Value::as_str)
            .unwrap_or("void")
            .into(),
    }
}

fn parse_bases(node: &Value) -> Vec<String> {
    node.get("bases")
        .and_then(Value::as_array)
        .map(|bases| {
            bases
                .iter()
                .filter_map(|base| base.get("type"))
                .filter_map(|ty| ty.get("qualType"))
                .filter_map(Value::as_str)
                .map(clean_cpp_type_name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn qualified_name(namespaces: &[String], name: &str) -> String {
    if namespaces.is_empty() {
        name.into()
    } else {
        format!("{}::{}", namespaces.join("::"), name)
    }
}

fn signature_key(name: &str, params: &[ParameterDecl]) -> String {
    let args = params.iter().map(|param| param.qual_type.as_str()).collect::<Vec<_>>();
    format!("{name}({})", args.join(","))
}

fn function_return_from_qualtype(qual_type: &str) -> String {
    // Find the function argument list by scanning for the last '(' at top-level
    // (i.e. not inside '<>' or '()'), so "std::function<int (int)> (int)" → "std::function<int (int)>"
    let bytes = qual_type.as_bytes();
    let mut paren_depth: i32 = 0;
    let mut angle_depth: i32 = 0;
    let mut last_top_open: Option<usize> = None;
    for (i, &ch) in bytes.iter().enumerate() {
        match ch {
            b'<' => angle_depth += 1,
            b'>' => {
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
            }
            b'(' if angle_depth == 0 => {
                if paren_depth == 0 {
                    last_top_open = Some(i);
                }
                paren_depth += 1;
            }
            b')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
            }
            _ => {}
        }
    }
    if let Some(pos) = last_top_open {
        qual_type[..pos].trim().to_string()
    } else {
        qual_type.trim().to_string()
    }
}

fn range(node: &Value) -> Option<SourceRange> {
    let begin = node
        .get("range")
        .and_then(|r| r.get("begin"))
        .and_then(|b| b.get("offset"))
        .and_then(Value::as_u64)?;
    let end = node
        .get("range")
        .and_then(|r| r.get("end"))
        .and_then(|b| b.get("offset"))
        .and_then(Value::as_u64)?;
    Some(SourceRange {
        begin: begin as usize,
        end: end as usize,
    })
}

fn source_slice(source: &str, range: Option<SourceRange>) -> String {
    if let Some(range) = range {
        let end = range.end.min(source.len());
        let begin = range.begin.min(end);
        source[begin..end].to_string()
    } else {
        String::new()
    }
}

fn kind(node: &Value) -> Option<&str> {
    node.get("kind").and_then(Value::as_str)
}

fn name(node: &Value) -> Option<&str> {
    node.get("name").and_then(Value::as_str)
}

fn children(node: &Value) -> &[Value] {
    node.get("inner")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Sentinel returned when a node has position info but no explicit file in its loc.
/// Clang AST JSON inherits the current file in context and only emits `loc.file`
/// when the file changes.  Nodes without a `file` key but with an `offset` or `line`
/// are therefore in the translation-unit root file.
const MAIN_TU_FILE: &str = "<main>";

fn node_location(node: &Value) -> Option<String> {
    let loc = node.get("loc")?;
    if let Some(file) = loc.get("file").and_then(Value::as_str) {
        return Some(file.into());
    }
    if let Some(file) = loc
        .get("includedFrom")
        .and_then(|value| value.get("file"))
        .and_then(Value::as_str)
    {
        return Some(file.into());
    }
    if let Some(file) = node
        .get("range")
        .and_then(|value| value.get("begin"))
        .and_then(|value| value.get("file"))
        .and_then(Value::as_str)
    {
        return Some(file.into());
    }
    // loc exists but no `file` key: the node is at an offset/line within the
    // currently-active file.  In the root TU that is the main source file.
    if loc.get("offset").is_some() || loc.get("line").is_some() {
        return Some(MAIN_TU_FILE.into());
    }
    None
}

fn is_system_path(path: &str) -> bool {
    path.contains("<built-in>")
        || path.contains("<command line>")
        || path.contains("libstdc++")
        || SYSTEM_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn normalize_path(path: &Path) -> String {
    relative_display(path)
}

fn normalize_loc(path: &str) -> String {
    path.replace('\\', "/")
}

fn is_methodish_name(name: &str) -> bool {
    matches!(name, "operator=" | "operator+" | "operator-" | "operator*" | "operator/")
}

fn clean_cpp_type_name(name: &str) -> String {
    name.replace("class ", "")
        .replace("struct ", "")
        .replace("const ", "")
        .replace(" &", "")
        .replace(" *", "")
        .trim()
        .to_string()
}

pub fn collect_feature_hints(unit: &TranslationUnit, source: &str) -> IndexMap<&'static str, bool> {
    let mut hints = IndexMap::new();
    hints.insert("operator_overload", unit.classes.iter().any(|class| class.methods.iter().any(|m| m.name.starts_with("operator"))));
    hints.insert("dynamic_cast", source.contains("dynamic_cast<"));
    hints.insert("placement_new", source.contains("placement new") || source.contains("new ("));
    hints.insert("std_function", source.contains("std::function<"));
    hints.insert(
        "stl_containers",
        source.contains("std::vector<") || source.contains("std::map<") || source.contains("std::set<"),
    );
    hints.insert("smart_pointers", source.contains("std::unique_ptr<") || source.contains("std::shared_ptr<"));
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/ast")
            .join(format!("{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn module_name_is_sanitized() {
        assert_eq!(module_name_from_source(Path::new("my-file.cpp")), "my_file");
    }

    #[test]
    fn file_matcher_accepts_file_name_suffix() {
        let dir = test_dir("matcher");
        let path = dir.join("main.cpp");
        fs::write(&path, "int add(int a, int b) { return a + b; }").unwrap();
        let matcher = FileMatcher::new(&path).unwrap();
        assert!(matcher.matches("main.cpp"));
    }

    #[test]
    fn parses_global_function_and_class() {
        let dir = test_dir("parse");
        let source = dir.join("main.cpp");
        fs::write(&source, "int add(int a, int b) { return a + b; } class Foo { public: int bar() const { return 1; } }; enum Color { RED = 0 }; typedef int MyInt;").unwrap();
        let ast = serde_json::json!({
            "kind": "TranslationUnitDecl",
            "inner": [
                {
                    "kind": "FunctionDecl",
                    "name": "add",
                    "loc": { "file": "main.cpp" },
                    "range": { "begin": { "offset": 0 }, "end": { "offset": 38 } },
                    "type": { "qualType": "int (int, int)" },
                    "inner": [
                        { "kind": "ParmVarDecl", "name": "a", "type": { "qualType": "int" } },
                        { "kind": "ParmVarDecl", "name": "b", "type": { "qualType": "int" } }
                    ]
                },
                {
                    "kind": "CXXRecordDecl",
                    "name": "Foo",
                    "tagUsed": "class",
                    "completeDefinition": true,
                    "loc": { "file": "main.cpp" },
                    "inner": [
                        {
                            "kind": "CXXMethodDecl",
                            "name": "bar",
                            "loc": { "file": "main.cpp" },
                            "type": { "qualType": "int () const" }
                        }
                    ]
                },
                {
                    "kind": "EnumDecl",
                    "name": "Color",
                    "loc": { "file": "main.cpp" },
                    "inner": [
                        { "kind": "EnumConstantDecl", "name": "RED", "inner": [{ "value": "0" }] }
                    ]
                },
                {
                    "kind": "TypedefDecl",
                    "name": "MyInt",
                    "loc": { "file": "main.cpp" },
                    "type": { "qualType": "int" }
                }
            ]
        });
        let ast_path = dir.join("main.cpp.json");
        fs::write(&ast_path, serde_json::to_string(&ast).unwrap()).unwrap();
        let unit = parse_translation_unit(&ast_path, &source).unwrap();
        assert_eq!(unit.functions.len(), 1);
        assert_eq!(unit.classes.len(), 1);
        assert_eq!(unit.enums.len(), 1);
        assert_eq!(unit.type_aliases.len(), 1);
    }

    #[test]
    fn collects_feature_hints_from_source() {
        let unit = TranslationUnit::default();
        let hints = collect_feature_hints(&unit, "dynamic_cast<Foo*>(bar); std::function<int(int)> f; new (buf) Foo();");
        assert_eq!(hints["dynamic_cast"], true);
        assert_eq!(hints["std_function"], true);
        assert_eq!(hints["placement_new"], true);
    }

    /// Clang AST JSON omits `loc.file` for nodes in the TU root file (only
    /// emits it when the file changes). This test verifies that such nodes
    /// are parsed correctly using the MAIN_TU_FILE sentinel.
    #[test]
    fn parses_functions_without_loc_file() {
        let dir = test_dir("no-loc-file");
        let source = dir.join("main.cpp");
        fs::write(&source, "int add(int a, int b) { return a + b; }").unwrap();
        // Simulate real Clang JSON: loc has offset/line but no "file" key.
        let ast = serde_json::json!({
            "kind": "TranslationUnitDecl",
            "inner": [
                {
                    "kind": "FunctionDecl",
                    "name": "add",
                    "loc": { "offset": 0, "line": 1, "col": 1, "tokLen": 3 },
                    "range": { "begin": { "offset": 0, "col": 1 }, "end": { "offset": 38, "col": 39 } },
                    "type": { "qualType": "int (int, int)" },
                    "inner": [
                        { "kind": "ParmVarDecl", "name": "a", "type": { "qualType": "int" } },
                        { "kind": "ParmVarDecl", "name": "b", "type": { "qualType": "int" } }
                    ]
                }
            ]
        });
        let ast_path = dir.join("main.cpp.json");
        fs::write(&ast_path, serde_json::to_string(&ast).unwrap()).unwrap();
        let unit = parse_translation_unit(&ast_path, &source).unwrap();
        assert_eq!(unit.functions.len(), 1, "function with offset-only loc must be captured");
        assert_eq!(unit.functions[0].name, "add");
    }
}
