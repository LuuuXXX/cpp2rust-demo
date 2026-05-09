//! Clang AST JSON parsing and C++ declaration extraction.
//!
//! This module:
//! 1. Defines Rust types that mirror the clang AST JSON structure.
//! 2. Extracts `FunctionIR` and `ClassIR` records from the JSON tree,
//!    filtered to only the declarations that come from the user-supplied
//!    header files (skipping system headers and implicit compiler-generated nodes).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Raw AST node types (deserialised from clang -ast-dump=json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AstNode {
    pub kind: String,
    pub id: Option<String>,
    pub loc: Option<Location>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_info: Option<TypeInfo>,
    #[serde(rename = "isImplicit")]
    pub is_implicit: Option<bool>,
    #[serde(rename = "isVirtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "isPure")]
    pub is_pure: Option<bool>,
    #[serde(rename = "storageClass")]
    pub storage_class: Option<String>,
    #[serde(rename = "completeDefinition")]
    pub complete_definition: Option<bool>,
    #[serde(rename = "tagUsed")]
    pub tag_used: Option<String>,
    pub access: Option<String>,
    pub inner: Option<Vec<AstNode>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub offset: Option<u64>,
    #[serde(rename = "spellingLoc")]
    pub spelling_loc: Option<Box<Location>>,
    #[serde(rename = "expansionLoc")]
    pub expansion_loc: Option<Box<Location>>,
    #[serde(rename = "includedFrom")]
    pub included_from: Option<Box<IncludedFrom>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IncludedFrom {
    pub file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeInfo {
    #[serde(rename = "qualType")]
    pub qual_type: String,
}

// ---------------------------------------------------------------------------
// Overload naming strategy
// ---------------------------------------------------------------------------

/// Strategy used to generate unique Rust names for C++ function overloads.
///
/// When multiple C++ functions share the same name (overloads), this strategy
/// determines how each overload is renamed in the generated Rust FFI.
///
/// # Extensibility
///
/// New variants can be added here to support additional naming schemes
/// (e.g. name-by-parameter-types, user-provided rename maps) without
/// changing the rest of the extraction pipeline.
#[derive(Debug, Clone, Default)]
pub enum OverloadStrategy {
    /// Append `_2`, `_3`, … to the second and subsequent overloads.
    ///
    /// The first occurrence keeps the plain snake_case name.
    /// This is the default and simplest strategy.
    #[default]
    NumericSuffix,
}

impl OverloadStrategy {
    /// Return the unique Rust name for a function.
    ///
    /// * `base` – the plain snake_case name derived from the C++ identifier.
    /// * `count` – 1-based count of how many times this overload key has
    ///   been seen (1 = first occurrence, 2 = second, …).
    pub fn uniquify(&self, base: &str, count: usize) -> String {
        match self {
            OverloadStrategy::NumericSuffix => {
                if count <= 1 {
                    base.to_string()
                } else {
                    format!("{}_{}", base, count)
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Intermediate representation (IR) – our cleaned-up model of C++ declarations
// ---------------------------------------------------------------------------

/// A single C++ function or method declaration.
#[derive(Debug, Clone)]
pub struct FunctionIR {
    /// Original C++ name (may be shared by overloads).
    pub name: String,
    /// Rust identifier (uniquified with numeric suffix for overloads).
    pub rust_name: String,
    /// C++ return type string.
    pub return_type: String,
    /// Rust return type string.
    pub rust_return_type: String,
    /// Parameter list.
    pub params: Vec<ParamIR>,
    /// Fully namespace-qualified C++ name, e.g. `"mylib::Widget::update"`.
    pub qualified_name: String,
    /// The string passed to `#[cpp(func = "...")]` or `#[cpp(method = "...")]`.
    pub cpp_signature: String,
    /// Whether this is a `const` method.
    pub is_const: bool,
    /// Whether this is a `static` method.
    pub is_static: bool,
    /// Whether this is a virtual method.
    pub is_virtual: bool,
    /// Whether this is a pure-virtual method.
    pub is_pure: bool,
    /// Class name, if this is a method.
    pub class_name: Option<String>,
}

/// A single function parameter.
#[derive(Debug, Clone)]
pub struct ParamIR {
    pub name: String,
    pub cpp_type: String,
    pub rust_type: String,
}

/// A C++ class or struct declaration.
#[derive(Debug, Clone)]
pub struct ClassIR {
    pub name: String,
    pub qualified_name: String,
    /// Public methods (constructors/destructors are excluded from hicc mappings).
    pub methods: Vec<FunctionIR>,
}

/// All declarations extracted from a set of header files.
#[derive(Debug, Default)]
pub struct ExtractedDecls {
    pub functions: Vec<FunctionIR>,
    pub classes: Vec<ClassIR>,
}

// ---------------------------------------------------------------------------
// Extraction logic
// ---------------------------------------------------------------------------

/// Run `clang -Xclang -ast-dump=json -fsyntax-only` on the given header and
/// return the parsed AST root node.
pub fn dump_ast(
    header: &Path,
    extra_clang_args: &[String],
    clang_bin: &str,
) -> crate::error::Result<AstNode> {
    use anyhow::anyhow;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(clang_bin);
    cmd.arg("-Xclang")
        .arg("-ast-dump=json")
        .arg("-fsyntax-only")
        .arg("-x")
        .arg("c++")
        .arg("-std=c++14");
    for arg in extra_clang_args {
        cmd.arg(arg);
    }
    cmd.arg(header);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| anyhow!("failed to run {}: {}", clang_bin, e))?;

    if output.stdout.is_empty() {
        return Err(anyhow!(
            "clang produced no AST output for {}",
            header.display()
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("parse clang AST JSON for {}: {}", header.display(), e))
}

/// Extract `FunctionIR` / `ClassIR` from the AST root, keeping only declarations
/// that originate in the given `target_files`.
///
/// The `strategy` parameter controls how overloaded function names are
/// disambiguated in the generated Rust FFI.
pub fn extract_declarations(
    ast_root: &AstNode,
    target_files: &[&Path],
) -> ExtractedDecls {
    extract_declarations_with_strategy(ast_root, target_files, &OverloadStrategy::default())
}

/// Same as [`extract_declarations`] but accepts an explicit overload naming
/// strategy.  Use this when you need to customise how overloads are renamed.
pub fn extract_declarations_with_strategy(
    ast_root: &AstNode,
    target_files: &[&Path],
    strategy: &OverloadStrategy,
) -> ExtractedDecls {
    // First pass: collect a global map of bare class names → qualified class names.
    // This is used to qualify parameter types that clang emits without their
    // namespace prefix (e.g., `Vec2 &` inside `namespace geo` should become
    // `geo::Vec2 &` in the generated C++ signature so hicc-build can resolve it).
    let class_name_map = collect_class_name_map(ast_root);

    let mut result = ExtractedDecls::default();
    let mut current_file = String::new();
    let mut overload_counts: HashMap<String, usize> = HashMap::new();

    walk_node(
        ast_root,
        &mut current_file,
        target_files,
        &[],
        &mut result,
        &mut overload_counts,
        strategy,
        &class_name_map,
    );

    result
}

// ---------------------------------------------------------------------------
// Class-name qualification helpers
// ---------------------------------------------------------------------------

/// Build a map from bare class name → fully-qualified class name by scanning
/// the entire AST (not just target files).  This lets us qualify parameter
/// types that clang emits without their namespace prefix.
///
/// When two classes share the same bare name (e.g. `A::Vec` and `B::Vec`),
/// the first one encountered in a depth-first, pre-order traversal is kept.
/// In cases of ambiguity, use the already-qualified form `ns::Vec` in your
/// C++ header so that clang emits it qualified.
fn collect_class_name_map(root: &AstNode) -> HashMap<String, String> {
    let mut map = HashMap::new();
    collect_class_names(root, &[], &mut map);
    map
}

fn collect_class_names(node: &AstNode, namespace: &[String], map: &mut HashMap<String, String>) {
    match node.kind.as_str() {
        "NamespaceDecl" => {
            if let Some(ref ns_name) = node.name {
                let mut ns = namespace.to_vec();
                ns.push(ns_name.clone());
                for child in node.inner.iter().flatten() {
                    collect_class_names(child, &ns, map);
                }
            }
        }
        "CXXRecordDecl" => {
            if node.complete_definition.unwrap_or(false) {
                if let Some(class_name) = node.name.as_deref() {
                    let qualified = make_qualified(namespace, class_name);
                    // First definition wins (forward decls are skipped above).
                    map.entry(class_name.to_string()).or_insert(qualified);
                    // Recurse for nested classes.
                    for child in node.inner.iter().flatten() {
                        collect_class_names(child, namespace, map);
                    }
                }
            }
        }
        _ => {
            for child in node.inner.iter().flatten() {
                collect_class_names(child, namespace, map);
            }
        }
    }
}

/// Qualify a C++ type string by replacing a bare class name with its
/// fully-qualified form, using `class_map` built from the AST.
///
/// Handles the common clang `qualType` patterns:
/// - `ClassName`        → `ns::ClassName`
/// - `const ClassName`  → `const ns::ClassName`
/// - `ClassName &`      → `ns::ClassName &`
/// - `const ClassName &`→ `const ns::ClassName &`
/// - `ClassName *`      → `ns::ClassName *`
/// - `const ClassName *`→ `const ns::ClassName *`
///
/// Already-qualified names (those containing `::`) are returned unchanged.
fn qualify_cpp_type(cpp_type: &str, class_map: &HashMap<String, String>) -> String {
    let trimmed = cpp_type.trim();

    // Strip trailing `&` or `*`.
    let (core, suffix) = if trimmed.ends_with(" &") || trimmed.ends_with('&') {
        let core = trimmed.trim_end_matches('&').trim_end();
        (core, " &")
    } else if trimmed.ends_with(" *") || trimmed.ends_with('*') {
        // Only handle single pointer (not double pointer **).
        // Double pointers (`ClassName **`) are left unchanged: hicc does not
        // support them directly, and they are already documented as a known
        // limitation.
        let without = trimmed.trim_end_matches('*').trim_end();
        if without.ends_with('*') {
            // Double pointer – leave as-is (known limitation, see docs/design.md).
            return cpp_type.to_string();
        }
        (without, " *")
    } else {
        (trimmed, "")
    };

    // Strip optional `const` prefix.
    let (is_const, bare) = if core.starts_with("const ") {
        (true, core["const ".len()..].trim())
    } else {
        (false, core)
    };

    // If already qualified, leave it alone.
    if bare.contains("::") {
        return cpp_type.to_string();
    }

    // Look up in class map.
    if let Some(qualified) = class_map.get(bare) {
        let const_prefix = if is_const { "const " } else { "" };
        format!("{}{}{}", const_prefix, qualified, suffix)
    } else {
        cpp_type.to_string()
    }
}

// ---------------------------------------------------------------------------
// Internal traversal
// ---------------------------------------------------------------------------

/// Update `current_file` from a `Location`, following expansion / spelling locs.
fn update_file(loc: &Location, current_file: &mut String) {
    if let Some(ref f) = loc.file {
        if !f.is_empty() {
            *current_file = f.clone();
            return;
        }
    }
    if let Some(ref exp) = loc.expansion_loc {
        update_file(exp, current_file);
    }
}

/// Check whether `file` is one of the user-supplied target headers.
fn is_target(file: &str, targets: &[&Path]) -> bool {
    if file.is_empty() {
        return false;
    }
    let p = Path::new(file);
    targets.iter().any(|t| {
        p == *t
            || p.canonicalize().ok().as_deref() == t.canonicalize().ok().as_deref()
            || p.file_name() == t.file_name()
    })
}

fn walk_node(
    node: &AstNode,
    current_file: &mut String,
    targets: &[&Path],
    namespace: &[String],
    result: &mut ExtractedDecls,
    overload_counts: &mut HashMap<String, usize>,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
) {
    // Advance current_file tracker.
    if let Some(ref loc) = node.loc {
        update_file(loc, current_file);
    }

    // Skip compiler-generated nodes.
    if node.is_implicit.unwrap_or(false) {
        return;
    }

    match node.kind.as_str() {
        "NamespaceDecl" => {
            if let Some(ref ns_name) = node.name {
                let mut ns = namespace.to_vec();
                ns.push(ns_name.clone());
                for child in node.inner.iter().flatten() {
                    walk_node(child, current_file, targets, &ns, result, overload_counts, strategy, class_map);
                }
            }
        }

        "FunctionDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            if let Some(ir) = extract_function(node, namespace, overload_counts, None, strategy, class_map) {
                result.functions.push(ir);
            }
        }

        "CXXRecordDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            // Only process complete definitions (not forward declarations).
            if !node.complete_definition.unwrap_or(false) {
                return;
            }
            let Some(class_name) = node.name.as_deref() else {
                return;
            };
            let qualified_name = make_qualified(namespace, class_name);
            let mut class_ir = ClassIR {
                name: class_name.to_string(),
                qualified_name,
                methods: vec![],
            };

            // C++ class members are private by default; struct members are public.
            let is_struct = node.tag_used.as_deref() == Some("struct");
            let mut cur_access = if is_struct { "public" } else { "private" };

            let mut method_overloads: HashMap<String, usize> = HashMap::new();

            for child in node.inner.iter().flatten() {
                if child.is_implicit.unwrap_or(false) {
                    continue;
                }
                // Track location inside the class body too.
                if let Some(ref loc) = child.loc {
                    update_file(loc, current_file);
                }

                if child.kind == "AccessSpecDecl" {
                    if let Some(ref a) = child.access {
                        cur_access = a.as_str();
                    }
                    continue;
                }

                // Only extract public members.
                if cur_access != "public" {
                    continue;
                }

                match child.kind.as_str() {
                    "CXXMethodDecl" | "CXXConstructorDecl" | "CXXDestructorDecl" => {
                        // Skip constructors and destructors – they need special hicc handling.
                        if matches!(child.kind.as_str(), "CXXConstructorDecl" | "CXXDestructorDecl")
                        {
                            continue;
                        }
                        if let Some(ir) =
                            extract_function(child, namespace, &mut method_overloads, Some(class_name), strategy, class_map)
                        {
                            class_ir.methods.push(ir);
                        }
                    }
                    _ => {}
                }
            }

            result.classes.push(class_ir);
        }

        // extern "C" / extern "C++" linkage blocks – just descend.
        "LinkageSpecDecl" => {
            for child in node.inner.iter().flatten() {
                walk_node(child, current_file, targets, namespace, result, overload_counts, strategy, class_map);
            }
        }

        // For any other node type, continue traversal so we don't miss
        // declarations inside (e.g. anonymous namespaces).
        _ => {
            for child in node.inner.iter().flatten() {
                walk_node(child, current_file, targets, namespace, result, overload_counts, strategy, class_map);
            }
        }
    }
}

/// Extract a `FunctionIR` from a `FunctionDecl`, `CXXMethodDecl`, etc.
fn extract_function(
    node: &AstNode,
    namespace: &[String],
    overload_counts: &mut HashMap<String, usize>,
    class_name: Option<&str>,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
) -> Option<FunctionIR> {
    let name = node.name.as_deref()?;

    // Destructors start with '~' – skip.
    if name.starts_with('~') {
        return None;
    }

    let qual_type = node.type_info.as_ref()?.qual_type.as_str();
    let (return_type, _) = parse_fn_qual_type(qual_type)?;

    // Collect parameters from ParmVarDecl children.
    let params: Vec<ParamIR> = node
        .inner
        .iter()
        .flatten()
        .filter(|c| c.kind == "ParmVarDecl")
        .enumerate()
        .map(|(i, p)| {
            let pname = p
                .name
                .as_deref()
                .filter(|n| !n.is_empty())
                .unwrap_or(&format!("arg{}", i))
                .to_string();
            let cpp_type = p
                .type_info
                .as_ref()
                .map(|t| t.qual_type.as_str())
                .unwrap_or("void")
                .to_string();
            let rust_type = cpp_to_rust_type(&cpp_type);
            ParamIR {
                name: pname,
                cpp_type,
                rust_type,
            }
        })
        .collect();

    let is_const = qual_type.ends_with(") const") || qual_type.ends_with("() const");
    let is_static = node.storage_class.as_deref() == Some("static");
    let is_virtual = node.is_virtual.unwrap_or(false);
    let is_pure = node.is_pure.unwrap_or(false);

    // Build the fully-qualified C++ name.
    let qualified_name = if let Some(cls) = class_name {
        let ns_part = make_qualified(namespace, cls);
        format!("{}::{}", ns_part, name)
    } else {
        make_qualified(namespace, name)
    };

    // Build the C++ signature for hicc attributes.
    // Qualify any bare class-type names with their namespace prefix so that
    // hicc-build can resolve them (clang often omits the namespace prefix for
    // types defined in the same namespace as the function).
    let qualified_return = qualify_cpp_type(&return_type, class_map);
    let param_types: Vec<String> = params
        .iter()
        .map(|p| qualify_cpp_type(&p.cpp_type, class_map))
        .collect();
    let const_suffix = if is_const { " const" } else { "" };
    let cpp_signature = format!(
        "{} {}({}){}",
        qualified_return,
        qualified_name,
        param_types.join(", "),
        const_suffix
    );

    // Overload resolution via the configured strategy.
    let overload_key = qualified_name.clone();
    let count = overload_counts.entry(overload_key).or_insert(0);
    *count += 1;
    let rust_name = strategy.uniquify(&to_snake_case(name), *count);

    let rust_return_type = cpp_to_rust_type(&return_type);

    Some(FunctionIR {
        name: name.to_string(),
        rust_name,
        return_type,
        rust_return_type,
        params,
        qualified_name,
        cpp_signature,
        is_const,
        is_static,
        is_virtual,
        is_pure,
        class_name: class_name.map(|s| s.to_string()),
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a clang function qualType like `"int (int, double) const"` into
/// `("int", "int, double")`.  Returns `None` if the string is not a function type.
fn parse_fn_qual_type(qual_type: &str) -> Option<(String, String)> {
    // The separator between return type and param list is always " (": a
    // space followed by an opening parenthesis.  The return type itself never
    // contains this pattern in practice (template args use `<`, not ` (`).
    let sep = qual_type.find(" (")?;
    let return_type = qual_type[..sep].trim().to_string();
    let after_open = &qual_type[sep + 2..]; // skip " ("
    let close = after_open.find(')')?;
    let params_str = after_open[..close].trim().to_string();
    Some((return_type, params_str))
}

/// Build `"ns1::ns2::name"` from a namespace slice and a leaf name.
fn make_qualified(namespace: &[String], name: &str) -> String {
    if namespace.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", namespace.join("::"), name)
    }
}

/// Convert a CamelCase / mixedCase C++ identifier to Rust snake_case.
pub fn to_snake_case(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next = chars.get(i + 1).copied();
                // Insert '_' when:
                //   (a) previous char was lowercase/digit → starting a new word, or
                //   (b) previous char was uppercase AND next char is lowercase
                //       → transitioning from an acronym to a word (e.g. "HTTPServer").
                if prev.is_ascii_lowercase()
                    || prev.is_ascii_digit()
                    || (prev.is_ascii_uppercase()
                        && next.map(|n| n.is_ascii_lowercase()).unwrap_or(false))
                {
                    out.push('_');
                }
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Map a C++ type string to a Rust type string suitable for hicc FFI.
///
/// This handles the common primitive types.  Complex types (user-defined
/// classes, templates, etc.) are passed through as the bare class name so
/// hicc can handle them via its `class` declarations.
pub fn cpp_to_rust_type(cpp_type: &str) -> String {
    let t = cpp_type.trim();

    // Pointer: "T *" or "T*" or "const T *" etc.
    // We look for a trailing `*` that is not part of `**`.
    if let Some(inner) = strip_trailing_ptr(t) {
        let is_const = inner.starts_with("const ");
        let base = inner.strip_prefix("const ").unwrap_or(inner).trim();
        if base == "void" {
            return if is_const {
                "*const core::ffi::c_void".to_string()
            } else {
                "*mut core::ffi::c_void".to_string()
            };
        }
        let rust_base = cpp_to_rust_type(base);
        return if is_const {
            format!("*const {}", rust_base)
        } else {
            format!("*mut {}", rust_base)
        };
    }

    // Reference: "T &" or "T&" or "const T &"
    if let Some(inner) = strip_trailing_ref(t) {
        let is_const = inner.starts_with("const ");
        let base = inner.strip_prefix("const ").unwrap_or(inner).trim();
        let rust_base = cpp_to_rust_type(base);
        return if is_const {
            format!("&{}", rust_base)
        } else {
            format!("&mut {}", rust_base)
        };
    }

    // Strip leading `const` for simple types.
    if let Some(rest) = t.strip_prefix("const ") {
        return cpp_to_rust_type(rest);
    }

    // Primitive mappings.
    match t {
        "void" => "()".to_string(),
        "bool" => "bool".to_string(),
        "char" | "signed char" => "i8".to_string(),
        "unsigned char" => "u8".to_string(),
        "short" | "short int" | "signed short" | "signed short int" => "i16".to_string(),
        "unsigned short" | "unsigned short int" => "u16".to_string(),
        "int" | "signed" | "signed int" => "i32".to_string(),
        "unsigned" | "unsigned int" => "u32".to_string(),
        "long" | "long int" | "signed long" | "signed long int" => "i64".to_string(),
        "unsigned long" | "unsigned long int" => "u64".to_string(),
        "long long" | "long long int" | "signed long long" => "i64".to_string(),
        "unsigned long long" | "unsigned long long int" => "u64".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "long double" => "f64".to_string(),
        "size_t" => "usize".to_string(),
        "ssize_t" => "isize".to_string(),
        "ptrdiff_t" => "isize".to_string(),
        "int8_t" => "i8".to_string(),
        "int16_t" => "i16".to_string(),
        "int32_t" => "i32".to_string(),
        "int64_t" => "i64".to_string(),
        "uint8_t" => "u8".to_string(),
        "uint16_t" => "u16".to_string(),
        "uint32_t" => "u32".to_string(),
        "uint64_t" => "u64".to_string(),
        "intptr_t" => "isize".to_string(),
        "uintptr_t" => "usize".to_string(),
        _ => {
            // For user-defined / STL types: strip namespace for the bare class name.
            // hicc will look up the Rust struct by the name declared with `class`.
            bare_class_name(t)
        }
    }
}

/// Strip a trailing `*` (with optional surrounding spaces) from a type string,
/// returning the inner type.  Does NOT strip double-pointers `**`.
fn strip_trailing_ptr(t: &str) -> Option<&str> {
    let trimmed = t.trim_end();
    if trimmed.ends_with('*') {
        let without = trimmed[..trimmed.len() - 1].trim_end();
        // Avoid stripping the second star of `**`.
        if without.ends_with('*') {
            return None;
        }
        Some(without)
    } else {
        None
    }
}

/// Strip a trailing `&` (with optional surrounding spaces) from a type string.
fn strip_trailing_ref(t: &str) -> Option<&str> {
    let trimmed = t.trim_end();
    if trimmed.ends_with('&') {
        Some(trimmed[..trimmed.len() - 1].trim_end())
    } else {
        None
    }
}

/// Extract the bare class name, dropping leading namespaces and template args.
///
/// Examples:
/// - `"std::vector<int>"` → `"vector"`
/// - `"mylib::Widget"` → `"Widget"`
/// - `"MyClass"` → `"MyClass"`
fn bare_class_name(t: &str) -> String {
    // Take the last `::` segment.
    let last = t.rsplit("::").next().unwrap_or(t).trim();
    // Drop template parameters.
    last.split('<').next().unwrap_or(last).trim().to_string()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fn_qual_type() {
        assert_eq!(
            parse_fn_qual_type("int (int, double)"),
            Some(("int".to_string(), "int, double".to_string()))
        );
        assert_eq!(
            parse_fn_qual_type("void ()"),
            Some(("void".to_string(), "".to_string()))
        );
        assert_eq!(
            parse_fn_qual_type("int () const"),
            Some(("int".to_string(), "".to_string()))
        );
        assert_eq!(
            parse_fn_qual_type("const char * (const char *, int)"),
            Some(("const char *".to_string(), "const char *, int".to_string()))
        );
    }

    #[test]
    fn test_cpp_to_rust_type_primitives() {
        assert_eq!(cpp_to_rust_type("int"), "i32");
        assert_eq!(cpp_to_rust_type("unsigned int"), "u32");
        assert_eq!(cpp_to_rust_type("double"), "f64");
        assert_eq!(cpp_to_rust_type("void"), "()");
        assert_eq!(cpp_to_rust_type("bool"), "bool");
        assert_eq!(cpp_to_rust_type("size_t"), "usize");
        assert_eq!(cpp_to_rust_type("int32_t"), "i32");
        assert_eq!(cpp_to_rust_type("uint64_t"), "u64");
    }

    #[test]
    fn test_cpp_to_rust_type_pointers() {
        assert_eq!(cpp_to_rust_type("const char *"), "*const i8");
        assert_eq!(cpp_to_rust_type("char *"), "*mut i8");
        assert_eq!(cpp_to_rust_type("void *"), "*mut core::ffi::c_void");
        assert_eq!(cpp_to_rust_type("const void *"), "*const core::ffi::c_void");
        assert_eq!(cpp_to_rust_type("int *"), "*mut i32");
    }

    #[test]
    fn test_cpp_to_rust_type_refs() {
        assert_eq!(cpp_to_rust_type("const int &"), "&i32");
        assert_eq!(cpp_to_rust_type("int &"), "&mut i32");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("add"), "add");
        assert_eq!(to_snake_case("getId"), "get_id");
        assert_eq!(to_snake_case("instanceCount"), "instance_count");
        assert_eq!(to_snake_case("Widget"), "widget");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
    }

    #[test]
    fn test_bare_class_name() {
        assert_eq!(bare_class_name("std::vector<int>"), "vector");
        assert_eq!(bare_class_name("mylib::Widget"), "Widget");
        assert_eq!(bare_class_name("MyClass"), "MyClass");
    }

    #[test]
    fn test_qualify_cpp_type_no_match() {
        let map = HashMap::from([("Vec2".to_string(), "geo::Vec2".to_string())]);
        // Primitives pass through unchanged.
        assert_eq!(qualify_cpp_type("int", &map), "int");
        assert_eq!(qualify_cpp_type("double", &map), "double");
        assert_eq!(qualify_cpp_type("const int &", &map), "const int &");
        assert_eq!(qualify_cpp_type("int *", &map), "int *");
    }

    #[test]
    fn test_qualify_cpp_type_bare() {
        let map = HashMap::from([("Vec2".to_string(), "geo::Vec2".to_string())]);
        assert_eq!(qualify_cpp_type("Vec2", &map), "geo::Vec2");
        assert_eq!(qualify_cpp_type("Vec2 &", &map), "geo::Vec2 &");
        assert_eq!(qualify_cpp_type("Vec2 *", &map), "geo::Vec2 *");
        assert_eq!(qualify_cpp_type("const Vec2 &", &map), "const geo::Vec2 &");
        assert_eq!(qualify_cpp_type("const Vec2 *", &map), "const geo::Vec2 *");
    }

    #[test]
    fn test_qualify_cpp_type_already_qualified() {
        let map = HashMap::from([("Vec2".to_string(), "geo::Vec2".to_string())]);
        // Already qualified names are left alone.
        assert_eq!(qualify_cpp_type("geo::Vec2 &", &map), "geo::Vec2 &");
        assert_eq!(qualify_cpp_type("other::Vec2 *", &map), "other::Vec2 *");
    }
}
