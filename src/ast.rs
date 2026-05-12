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
    /// The AST node kind as emitted by clang (e.g. `"FunctionDecl"`,
    /// `"CXXRecordDecl"`, …).  Some internal clang nodes lack this field
    /// (e.g. certain template-expansion helpers); defaulting to an empty
    /// string ensures they are silently skipped during traversal rather than
    /// causing a hard deserialisation failure.
    #[serde(default)]
    pub kind: String,
    pub id: Option<String>,
    pub loc: Option<Location>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_info: Option<TypeInfo>,
    #[serde(rename = "isImplicit")]
    pub is_implicit: Option<bool>,
    // Clang JSON uses either `isVirtual`/`isPure` or `virtual`/`pure`
    // depending on version/output mode, so accept both spellings.
    #[serde(rename = "isVirtual", alias = "virtual")]
    pub is_virtual: Option<bool>,
    #[serde(rename = "isPure", alias = "pure")]
    pub is_pure: Option<bool>,
    #[serde(rename = "storageClass")]
    pub storage_class: Option<String>,
    #[serde(rename = "completeDefinition")]
    pub complete_definition: Option<bool>,
    #[serde(rename = "tagUsed")]
    pub tag_used: Option<String>,
    pub access: Option<String>,
    pub inner: Option<Vec<AstNode>>,
    /// Direct base class specifiers for `CXXRecordDecl` nodes.
    ///
    /// Clang emits these as a top-level `"bases"` array on the record node,
    /// **not** as children inside `"inner"`.  Each entry has an `access` field
    /// (`"public"`, `"protected"`, `"private"`) and a `type.qualType` with the
    /// base class name.
    #[serde(default)]
    pub bases: Vec<BaseSpecifier>,
}

/// A direct base class entry as emitted in clang's `"bases"` array.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseSpecifier {
    pub access: String,
    #[serde(rename = "type")]
    pub type_info: TypeInfo,
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

/// A public constructor extracted from a C++ class.
///
/// hicc uses this via `#[cpp(class = "...", ctor = "...")]` to let Rust
/// construct the C++ object directly (no separate factory function needed).
#[derive(Debug, Clone)]
pub struct CtorIR {
    /// Parameter list (same type as method params).
    pub params: Vec<ParamIR>,
    /// The string placed in `ctor = "..."`, e.g. `"Widget(int, double)"`.
    pub cpp_signature: String,
}

/// A C++ global variable that can be accessed from Rust via
/// `#[cpp(data = "qualified_name")]` in `hicc::import_lib!`.
#[derive(Debug, Clone)]
pub struct GlobalVarIR {
    /// Bare C++ identifier.
    pub name: String,
    /// Fully namespace-qualified name, e.g. `"myns::g_counter"`.
    pub qualified_name: String,
    /// C++ type string as emitted by clang.
    pub cpp_type: String,
    /// Mapped Rust type string.
    pub rust_type: String,
    /// Whether the variable is `const`-qualified.
    pub is_const: bool,
}

/// A C++ class or struct declaration.
#[derive(Debug, Clone)]
pub struct ClassIR {
    pub name: String,
    pub qualified_name: String,
    /// Public methods (constructors/destructors are excluded from hicc mappings).
    pub methods: Vec<FunctionIR>,
    /// `true` when every public method is pure-virtual.
    ///
    /// A fully-abstract class (all methods are `= 0`) maps to a hicc
    /// `#[interface]` trait rather than a concrete `#[cpp(class = "...")]`
    /// struct.  Non-pure virtual methods on non-abstract classes are extracted
    /// normally; hicc calls them through the C++ vtable transparently.
    pub is_abstract: bool,
    /// Public constructors extracted from this class.
    ///
    /// Empty when no usable constructor was found (e.g. all constructors are
    /// copy/move/implicit).  When non-empty the first entry is used as the
    /// primary `ctor = "..."` in `import_class!`; additional entries are
    /// exposed as factory functions via `#[member(class = ..., method = "new_N")]`
    /// in `import_lib!`.
    pub ctors: Vec<CtorIR>,
    /// Names of direct public base classes, in declaration order.
    ///
    /// Used to generate the `class Foo: Base1, Base2 { ... }` syntax in
    /// `import_class!` so hicc knows the inheritance chain.
    pub bases: Vec<String>,
}

/// A declaration that is intentionally skipped during extraction.
#[derive(Debug, Clone)]
pub struct SkippedDecl {
    /// AST node kind, e.g. `CXXMethodDecl`.
    pub kind: String,
    /// Name or qualified name when available.
    pub name: String,
    /// Stable skip reason key.
    pub reason: String,
}

/// All declarations extracted from a set of header files.
#[derive(Debug, Default)]
pub struct ExtractedDecls {
    pub functions: Vec<FunctionIR>,
    pub classes: Vec<ClassIR>,
    pub globals: Vec<GlobalVarIR>,
    pub skipped: Vec<SkippedDecl>,
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
pub fn extract_declarations(ast_root: &AstNode, target_files: &[&Path]) -> ExtractedDecls {
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
                    walk_node(
                        child,
                        current_file,
                        targets,
                        &ns,
                        result,
                        overload_counts,
                        strategy,
                        class_map,
                    );
                }
            }
        }

        "FunctionDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            if is_operator_name(node.name.as_deref()) {
                record_skipped(result, node, namespace, None, "operator_overload");
                return;
            }
            if let Some(ir) = extract_function(
                node,
                namespace,
                overload_counts,
                None,
                strategy,
                class_map,
                &mut result.skipped,
            ) {
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
                is_abstract: false,
                ctors: vec![],
                bases: vec![],
            };

            // Extract public base classes from the top-level `bases` array
            // (clang emits base-class specifiers here, NOT in `inner`).
            for base in &node.bases {
                if base.access == "public" {
                    let raw = base.type_info.qual_type.trim();
                    let bare = raw
                        .strip_prefix("class ")
                        .or_else(|| raw.strip_prefix("struct "))
                        .unwrap_or(raw)
                        .trim();
                    if !bare.is_empty() {
                        let qualified_base = class_map
                            .get(bare)
                            .cloned()
                            .unwrap_or_else(|| bare.to_string());
                        class_ir.bases.push(qualified_base);
                    }
                }
            }

            // C++ class members are private by default; struct members are public.
            let is_struct = node.tag_used.as_deref() == Some("struct");
            let mut cur_access = if is_struct { "public" } else { "private" };

            let mut method_overloads: HashMap<String, usize> = HashMap::new();
            // Collect pure-virtual methods separately; we decide after the loop
            // whether this class is fully abstract (→ #[interface]) or mixed.
            let mut pure_virtual_nodes: Vec<AstNode> = Vec::new();

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
                        if child.kind == "CXXDestructorDecl" {
                            record_skipped(result, child, namespace, Some(class_name), "destructor");
                            continue;
                        }
                        if child.kind == "CXXConstructorDecl" {
                            // Attempt to extract this constructor for hicc ctor support.
                            // Skip copy/move constructors (single parameter of `const T &`
                            // or `T &&` pattern) — these are compiler-generated plumbing,
                            // not user-facing factory constructors.
                            if let Some(ctor) = extract_ctor(child, class_name, class_map) {
                                class_ir.ctors.push(ctor);
                            } else {
                                record_skipped(
                                    result,
                                    child,
                                    namespace,
                                    Some(class_name),
                                    "constructor",
                                );
                            }
                            continue;
                        }
                        // Pure-virtual methods are held aside; we decide below whether
                        // the class is fully abstract or mixed.
                        if child.is_pure.unwrap_or(false) {
                            pure_virtual_nodes.push(child.clone());
                            continue;
                        }
                        // Non-pure virtual and regular methods: both are callable via
                        // hicc's extern-C adapters (C++ vtable dispatch is transparent).
                        if is_operator_name(child.name.as_deref()) {
                            record_skipped(
                                result,
                                child,
                                namespace,
                                Some(class_name),
                                "operator_overload",
                            );
                            continue;
                        }
                        if let Some(ir) = extract_function(
                            child,
                            namespace,
                            &mut method_overloads,
                            Some(class_name),
                            strategy,
                            class_map,
                            &mut result.skipped,
                        ) {
                            class_ir.methods.push(ir);
                        }
                    }
                    _ => {}
                }
            }

            // Decide how to handle pure-virtual methods.
            // If the class has *only* pure-virtual public methods (no concrete
            // methods extracted above), it is fully abstract and should be
            // emitted as a hicc `#[interface]` trait.
            let is_abstract = class_ir.methods.is_empty() && !pure_virtual_nodes.is_empty();
            if is_abstract {
                for pvm in &pure_virtual_nodes {
                    if is_operator_name(pvm.name.as_deref()) {
                        record_skipped(result, pvm, namespace, Some(class_name), "operator_overload");
                        continue;
                    }
                    if let Some(ir) = extract_function(
                        pvm,
                        namespace,
                        &mut method_overloads,
                        Some(class_name),
                        strategy,
                        class_map,
                        &mut result.skipped,
                    ) {
                        class_ir.methods.push(ir);
                    }
                }
            } else {
                // Mixed or non-abstract class: skip pure-virtual methods conservatively.
                for pvm in &pure_virtual_nodes {
                    record_skipped(result, pvm, namespace, Some(class_name), "pure_virtual");
                }
            }
            class_ir.is_abstract = is_abstract;

            result.classes.push(class_ir);
        }

        // extern "C" / extern "C++" linkage blocks – just descend.
        "LinkageSpecDecl" => {
            for child in node.inner.iter().flatten() {
                walk_node(
                    child,
                    current_file,
                    targets,
                    namespace,
                    result,
                    overload_counts,
                    strategy,
                    class_map,
                );
            }
        }
        "ClassTemplateDecl" | "FunctionTemplateDecl" | "ClassTemplateSpecializationDecl" => {
            if is_target(current_file, targets) {
                record_skipped(result, node, namespace, None, "template_decl");
            }
        }

        "VarDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            // Only extract non-static (i.e. namespace-scope / file-scope) variables.
            // `static` storage class here means a local static, not a global; skip those.
            if node.storage_class.as_deref() == Some("static") {
                return;
            }
            if let Some(global) = extract_global_var(node, namespace, class_map) {
                result.globals.push(global);
            }
        }

        // For any other node type, continue traversal so we don't miss
        // declarations inside (e.g. anonymous namespaces).
        _ => {
            for child in node.inner.iter().flatten() {
                walk_node(
                    child,
                    current_file,
                    targets,
                    namespace,
                    result,
                    overload_counts,
                    strategy,
                    class_map,
                );
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
    skipped: &mut Vec<SkippedDecl>,
) -> Option<FunctionIR> {
    let name = node.name.as_deref()?;
    let qualified_name = make_function_qualified_name(namespace, class_name, name);

    // Destructors start with '~' – skip.
    if name.starts_with('~') {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "destructor".to_string(),
        });
        return None;
    }

    if is_operator_name(Some(name)) {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "operator_overload".to_string(),
        });
        return None;
    }

    let Some(qual_type) = node.type_info.as_ref().map(|t| t.qual_type.as_str()) else {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
        });
        return None;
    };
    let Some((return_type, _)) = parse_fn_qual_type(qual_type) else {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
        });
        return None;
    };
    if !is_supported_cpp_type(&return_type, class_map) {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
        });
        return None;
    }

    // Collect parameters from ParmVarDecl children.
    let mut params: Vec<ParamIR> = Vec::new();
    for (i, p) in node
        .inner
        .iter()
        .flatten()
        .filter(|c| c.kind == "ParmVarDecl")
        .enumerate()
    {
        let pname = p
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(&format!("arg{}", i))
            .to_string();
        let Some(cpp_type) = p.type_info.as_ref().map(|t| t.qual_type.clone()) else {
            skipped.push(SkippedDecl {
                kind: node.kind.clone(),
                name: qualified_name.clone(),
                reason: "unsupported_type".to_string(),
            });
            return None;
        };
        if !is_supported_cpp_type(&cpp_type, class_map) {
            skipped.push(SkippedDecl {
                kind: node.kind.clone(),
                name: qualified_name.clone(),
                reason: "unsupported_type".to_string(),
            });
            return None;
        }
        let rust_type = cpp_to_rust_type(&cpp_type);
        params.push(ParamIR {
            name: pname,
            cpp_type,
            rust_type,
        });
    }

    let is_const = qual_type.ends_with(") const") || qual_type.ends_with("() const");
    let is_static = node.storage_class.as_deref() == Some("static");
    let is_virtual = node.is_virtual.unwrap_or(false);
    let is_pure = node.is_pure.unwrap_or(false);

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

fn make_function_qualified_name(
    namespace: &[String],
    class_name: Option<&str>,
    name: &str,
) -> String {
    if let Some(cls) = class_name {
        format!("{}::{}", make_qualified(namespace, cls), name)
    } else {
        make_qualified(namespace, name)
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

    // Pointer chain: supports single and multi-level pointers, e.g.
    // `int **`, `const char **`, `const T * const *`.
    if t.contains('*') && !t.contains("(*)") {
        let parts: Vec<&str> = t.split('*').collect();
        let ptr_depth = parts.len().saturating_sub(1);
        if ptr_depth > 0 {
            let base_part = parts[0].trim();
            let ptr_qualifiers: Vec<&str> = parts[1..].iter().map(|p| p.trim()).collect();
            let base_const = has_top_level_const(base_part);
            let base = strip_top_level_const(base_part);
            let mut rust_type = if base == "void" {
                "core::ffi::c_void".to_string()
            } else {
                cpp_to_rust_type(base)
            };
            for level in 0..ptr_depth {
                let pointee_const = if level == 0 {
                    base_const
                } else {
                    has_const_token(ptr_qualifiers[level - 1])
                };
                rust_type = if pointee_const {
                    format!("*const {}", rust_type)
                } else {
                    format!("*mut {}", rust_type)
                };
            }
            return rust_type;
        }
    }

    // Reference: "T &" or "T&" or "const T &"
    if let Some(inner) = strip_trailing_ref(t) {
        let is_const = has_top_level_const(inner);
        let base = strip_top_level_const(inner);
        let rust_base = cpp_to_rust_type(base);
        return if is_const {
            format!("&{}", rust_base)
        } else {
            format!("&mut {}", rust_base)
        };
    }

    // Strip top-level `const` for simple types.
    if has_top_level_const(t) {
        let rest = strip_top_level_const(t);
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

/// Strip a trailing `&` (with optional surrounding spaces) from a type string.
fn strip_trailing_ref(t: &str) -> Option<&str> {
    let trimmed = t.trim_end();
    if trimmed.ends_with('&') {
        Some(trimmed[..trimmed.len() - 1].trim_end())
    } else {
        None
    }
}

fn has_const_token(segment: &str) -> bool {
    segment.split_whitespace().any(|tok| tok == "const")
}

fn has_top_level_const(t: &str) -> bool {
    let trimmed = t.trim();
    trimmed.starts_with("const ") || trimmed.ends_with(" const") || trimmed == "const"
}

fn strip_top_level_const(t: &str) -> &str {
    let mut trimmed = t.trim();
    if let Some(rest) = trimmed.strip_prefix("const ") {
        trimmed = rest.trim_start();
    }
    if let Some(rest) = trimmed.strip_suffix(" const") {
        trimmed = rest.trim_end();
    }
    trimmed
}

fn is_operator_name(name: Option<&str>) -> bool {
    name.is_some_and(|n| n.starts_with("operator"))
}

fn is_supported_cpp_type(cpp_type: &str, class_map: &HashMap<String, String>) -> bool {
    let t = cpp_type.trim();
    if t.is_empty() {
        return false;
    }
    if contains_unsupported_type_construct(t) {
        return false;
    }

    if t.contains('*') {
        if t.contains("(*)") || t.contains("(&)") {
            return false;
        }
        let base = t
            .split('*')
            .next()
            .expect("split always has at least one element")
            .trim();
        let base = strip_top_level_const(base);
        return is_supported_cpp_type(base, class_map);
    }

    if let Some(inner) = strip_trailing_ref(t) {
        let base = strip_top_level_const(inner);
        return is_supported_cpp_type(base, class_map);
    }

    let base = strip_top_level_const(t);
    is_primitive_cpp_type(base) || is_known_class_type(base, class_map)
}

fn contains_unsupported_type_construct(t: &str) -> bool {
    t.contains('<')
        || t.contains('>')
        || t.contains('[')
        || t.contains(']')
        || t.contains("(*)")
        || t.contains("(&)")
        || t.contains("type-parameter-")
        || t.contains("dependent")
        || t.contains("decltype")
        || t == "auto"
}

fn is_primitive_cpp_type(t: &str) -> bool {
    matches!(
        t,
        "void"
            | "bool"
            | "char"
            | "signed char"
            | "unsigned char"
            | "short"
            | "short int"
            | "signed short"
            | "signed short int"
            | "unsigned short"
            | "unsigned short int"
            | "int"
            | "signed"
            | "signed int"
            | "unsigned"
            | "unsigned int"
            | "long"
            | "long int"
            | "signed long"
            | "signed long int"
            | "unsigned long"
            | "unsigned long int"
            | "long long"
            | "long long int"
            | "signed long long"
            | "unsigned long long"
            | "unsigned long long int"
            | "float"
            | "double"
            | "long double"
            | "size_t"
            | "ssize_t"
            | "ptrdiff_t"
            | "int8_t"
            | "int16_t"
            | "int32_t"
            | "int64_t"
            | "uint8_t"
            | "uint16_t"
            | "uint32_t"
            | "uint64_t"
            | "intptr_t"
            | "uintptr_t"
    )
}

fn is_known_class_type(t: &str, class_map: &HashMap<String, String>) -> bool {
    if class_map.contains_key(t) {
        return true;
    }
    let bare = bare_class_name(t);
    if class_map.contains_key(&bare) {
        return true;
    }
    class_map.values().any(|q| q == t)
}

fn record_skipped(
    result: &mut ExtractedDecls,
    node: &AstNode,
    namespace: &[String],
    class_name: Option<&str>,
    reason: &str,
) {
    let raw_name = node.name.as_deref().unwrap_or("<anonymous>");
    let name = if let Some(cls) = class_name {
        format!("{}::{}", make_qualified(namespace, cls), raw_name)
    } else {
        make_qualified(namespace, raw_name)
    };
    result.skipped.push(SkippedDecl {
        kind: node.kind.clone(),
        name,
        reason: reason.to_string(),
    });
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

/// Extract a `CtorIR` from a `CXXConstructorDecl` node.
///
/// Returns `None` when the constructor is copy/move (single param of type
/// `const ClassName &` or `ClassName &&`) or when any parameter has an
/// unsupported type.  All other user-defined constructors are extracted.
fn extract_ctor(
    node: &AstNode,
    class_name: &str,
    class_map: &HashMap<String, String>,
) -> Option<CtorIR> {
    // Collect ParmVarDecl children.
    let parm_nodes: Vec<&AstNode> = node
        .inner
        .iter()
        .flatten()
        .filter(|c| c.kind == "ParmVarDecl")
        .collect();

    // Detect copy/move constructor: single parameter whose type is
    // `const ClassName &` (copy) or `ClassName &&` (move).
    if parm_nodes.len() == 1 {
        if let Some(ref ti) = parm_nodes[0].type_info {
            let qt = ti.qual_type.trim();
            let bare = bare_class_name(class_name);
            // Copy: `const ClassName &`
            let is_copy = qt == format!("const {} &", bare)
                || qt == format!("const {} &", class_name);
            // Move: `ClassName &&`
            let is_move =
                qt == format!("{} &&", bare) || qt == format!("{} &&", class_name);
            if is_copy || is_move {
                return None;
            }
        }
    }

    let mut params: Vec<ParamIR> = Vec::new();
    for (i, p) in parm_nodes.iter().enumerate() {
        let pname = p
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(&format!("arg{}", i))
            .to_string();
        let Some(ref cpp_type_str) = p.type_info.as_ref().map(|t| t.qual_type.clone()) else {
            return None;
        };
        if !is_supported_cpp_type(cpp_type_str, class_map) {
            return None;
        }
        let rust_type = cpp_to_rust_type(cpp_type_str);
        params.push(ParamIR {
            name: pname,
            cpp_type: cpp_type_str.clone(),
            rust_type,
        });
    }

    let param_types: Vec<String> = params
        .iter()
        .map(|p| qualify_cpp_type(&p.cpp_type, class_map))
        .collect();
    let cpp_signature = format!("{}({})", class_name, param_types.join(", "));

    Some(CtorIR {
        params,
        cpp_signature,
    })
}

/// Extract a `GlobalVarIR` from a `VarDecl` node.
///
/// Returns `None` when the variable's type is unsupported (e.g. template
/// instantiation, function pointer, etc.).
fn extract_global_var(
    node: &AstNode,
    namespace: &[String],
    class_map: &HashMap<String, String>,
) -> Option<GlobalVarIR> {
    let name = node.name.as_deref()?.to_string();
    // Skip anonymous variables.
    if name.is_empty() || name == "<anonymous>" {
        return None;
    }

    let cpp_type = node.type_info.as_ref().map(|t| t.qual_type.clone())?;

    // Skip unsupported types (templates, function pointers, etc.).
    if !is_supported_cpp_type(&cpp_type, class_map) {
        return None;
    }

    let is_const = has_top_level_const(&cpp_type);
    let rust_type = if is_const {
        format!("&'static {}", cpp_to_rust_type(strip_top_level_const(&cpp_type)))
    } else {
        format!("&'static mut {}", cpp_to_rust_type(&cpp_type))
    };

    let qualified_name = make_qualified(namespace, &name);

    Some(GlobalVarIR {
        name,
        qualified_name,
        cpp_type,
        rust_type,
        is_const,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

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
        assert_eq!(cpp_to_rust_type("int **"), "*mut *mut i32");
        assert_eq!(cpp_to_rust_type("const char **"), "*mut *const i8");
        assert_eq!(cpp_to_rust_type("const char * const *"), "*const *const i8");
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

    #[test]
    fn test_is_supported_cpp_type() {
        let map = HashMap::from([("Vec2".to_string(), "geo::Vec2".to_string())]);
        assert!(is_supported_cpp_type("int", &map));
        assert!(is_supported_cpp_type("const Vec2 *", &map));
        assert!(is_supported_cpp_type("geo::Vec2 &", &map));
        assert!(!is_supported_cpp_type("std::vector<int> *", &map));
        assert!(!is_supported_cpp_type("Vec2 (*)(int)", &map));
        assert!(!is_supported_cpp_type("Document *", &map));
    }

    #[test]
    fn test_unsupported_type_helpers() {
        let map = HashMap::from([("Widget".to_string(), "ns::Widget".to_string())]);
        assert!(contains_unsupported_type_construct("std::vector<int>"));
        assert!(contains_unsupported_type_construct("int (*)()"));
        assert!(!contains_unsupported_type_construct("const Widget *"));

        assert!(is_primitive_cpp_type("int"));
        assert!(is_primitive_cpp_type("uint64_t"));
        assert!(!is_primitive_cpp_type("std::string"));

        assert!(is_known_class_type("Widget", &map));
        assert!(is_known_class_type("ns::Widget", &map));
        assert!(!is_known_class_type("Document", &map));
    }

    #[test]
    fn test_extract_skips_virtual_operator_and_templates() {
        let target = Path::new("/tmp/demo.cpp");
        let loc = Location {
            file: Some(target.display().to_string()),
            line: None,
            col: None,
            offset: None,
            spelling_loc: None,
            expansion_loc: None,
            included_from: None,
        };
        let ast = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            id: None,
            loc: Some(loc.clone()),
            name: None,
            type_info: None,
            is_implicit: None,
            is_virtual: None,
            is_pure: None,
            storage_class: None,
            complete_definition: None,
            tag_used: None,
            access: None,
            bases: vec![],
            inner: Some(vec![
                AstNode {
                    kind: "ClassTemplateDecl".to_string(),
                    id: None,
                    loc: Some(loc.clone()),
                    name: Some("Box".to_string()),
                    type_info: None,
                    is_implicit: None,
                    is_virtual: None,
                    is_pure: None,
                    storage_class: None,
                    complete_definition: None,
                    tag_used: None,
                    access: None,
                    bases: vec![],
                    inner: None,
                },
                AstNode {
                    kind: "FunctionDecl".to_string(),
                    id: None,
                    loc: Some(loc.clone()),
                    name: Some("operator+".to_string()),
                    type_info: Some(TypeInfo {
                        qual_type: "int (int, int)".to_string(),
                    }),
                    is_implicit: None,
                    is_virtual: None,
                    is_pure: None,
                    storage_class: None,
                    complete_definition: None,
                    tag_used: None,
                    access: None,
                    bases: vec![],
                    inner: Some(vec![]),
                },
                AstNode {
                    kind: "CXXRecordDecl".to_string(),
                    id: None,
                    loc: Some(loc.clone()),
                    name: Some("Widget".to_string()),
                    type_info: None,
                    is_implicit: None,
                    is_virtual: None,
                    is_pure: None,
                    storage_class: None,
                    complete_definition: Some(true),
                    tag_used: Some("class".to_string()),
                    access: None,
                    bases: vec![],
                    inner: Some(vec![
                        AstNode {
                            kind: "AccessSpecDecl".to_string(),
                            id: None,
                            loc: Some(loc.clone()),
                            name: None,
                            type_info: None,
                            is_implicit: None,
                            is_virtual: None,
                            is_pure: None,
                            storage_class: None,
                            complete_definition: None,
                            tag_used: None,
                            access: Some("public".to_string()),
                            bases: vec![],
                    inner: None,
                        },
                        AstNode {
                            kind: "CXXConstructorDecl".to_string(),
                            id: None,
                            loc: Some(loc.clone()),
                            name: Some("Widget".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "void ()".to_string(),
                            }),
                            is_implicit: None,
                            is_virtual: None,
                            is_pure: None,
                            storage_class: None,
                            complete_definition: None,
                            tag_used: None,
                            access: None,
                            bases: vec![],
                            inner: Some(vec![]),
                        },
                        AstNode {
                            kind: "CXXMethodDecl".to_string(),
                            id: None,
                            loc: Some(loc.clone()),
                            name: Some("virt".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "int ()".to_string(),
                            }),
                            is_implicit: None,
                            is_virtual: Some(true),
                            is_pure: Some(true),
                            storage_class: None,
                            complete_definition: None,
                            tag_used: None,
                            access: None,
                            bases: vec![],
                            inner: Some(vec![]),
                        },
                        AstNode {
                            kind: "CXXMethodDecl".to_string(),
                            id: None,
                            loc: Some(loc),
                            name: Some("operator[]".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "int (int) const".to_string(),
                            }),
                            is_implicit: None,
                            is_virtual: Some(false),
                            is_pure: Some(false),
                            storage_class: None,
                            complete_definition: None,
                            tag_used: None,
                            access: None,
                            bases: vec![],
                            inner: Some(vec![]),
                        },
                    ]),
                },
            ]),
        };

        let decls = extract_declarations(&ast, &[target]);
        assert!(decls.functions.is_empty());
        assert_eq!(decls.classes.len(), 1);
        // Widget is fully abstract (its only non-operator method is pure-virtual),
        // so the pure-virtual method is extracted and is_abstract is set.
        assert!(decls.classes[0].is_abstract);
        assert_eq!(decls.classes[0].methods.len(), 1);
        assert_eq!(decls.classes[0].methods[0].name, "virt");
        // The default constructor (0 params) is now extracted as a CtorIR, not skipped.
        assert_eq!(decls.classes[0].ctors.len(), 1, "Widget() should be extracted as a CtorIR");
        let reasons: Vec<&str> = decls.skipped.iter().map(|s| s.reason.as_str()).collect();
        assert!(reasons.contains(&"template_decl"));
        assert!(reasons.contains(&"operator_overload"));
        // constructor is NOT in skipped – it was extracted as CtorIR.
        assert!(!reasons.contains(&"constructor"), "extracted ctors should not appear in skipped list");
        // pure_virtual is NOT in skipped for fully-abstract classes (it was extracted).
        assert!(!reasons.contains(&"pure_virtual"));
    }

    /// Verify that a class with MIXED concrete + pure-virtual methods:
    /// - extracts the concrete methods normally (including non-pure virtual),
    /// - skips the pure-virtual methods (conservative), and
    /// - is NOT marked abstract.
    #[test]
    fn test_extract_mixed_virtual_class() {
        let target = Path::new("/tmp/mixed.cpp");
        let loc = Location {
            file: Some(target.display().to_string()),
            line: None,
            col: None,
            offset: None,
            spelling_loc: None,
            expansion_loc: None,
            included_from: None,
        };
        // Build a class with:
        //   public:
        //     virtual int virt_concrete();   ← non-pure virtual → extract
        //     virtual int pure_one() = 0;   ← pure-virtual in mixed class → skip
        //     int regular();                ← regular → extract
        let make_method = |name: &str, is_virtual: bool, is_pure: bool| AstNode {
            kind: "CXXMethodDecl".to_string(),
            id: None,
            loc: Some(loc.clone()),
            name: Some(name.to_string()),
            type_info: Some(TypeInfo { qual_type: "int ()".to_string() }),
            is_implicit: None,
            is_virtual: Some(is_virtual),
            is_pure: Some(is_pure),
            storage_class: None,
            complete_definition: None,
            tag_used: None,
            access: None,
            bases: vec![],
            inner: Some(vec![]),
        };
        let ast = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            id: None,
            loc: Some(loc.clone()),
            name: None,
            type_info: None,
            is_implicit: None,
            is_virtual: None,
            is_pure: None,
            storage_class: None,
            complete_definition: None,
            tag_used: None,
            access: None,
            bases: vec![],
            inner: Some(vec![AstNode {
                kind: "CXXRecordDecl".to_string(),
                id: None,
                loc: Some(loc.clone()),
                name: Some("Mixed".to_string()),
                type_info: None,
                is_implicit: None,
                is_virtual: None,
                is_pure: None,
                storage_class: None,
                complete_definition: Some(true),
                tag_used: Some("class".to_string()),
                access: None,
                bases: vec![],
                inner: Some(vec![
                    AstNode {
                        kind: "AccessSpecDecl".to_string(),
                        id: None,
                        loc: Some(loc.clone()),
                        name: None,
                        type_info: None,
                        is_implicit: None,
                        is_virtual: None,
                        is_pure: None,
                        storage_class: None,
                        complete_definition: None,
                        tag_used: None,
                        access: Some("public".to_string()),
                        bases: vec![],
                    inner: None,
                    },
                    make_method("virt_concrete", true, false),
                    make_method("pure_one", true, true),
                    make_method("regular", false, false),
                ]),
            }]),
        };

        let decls = extract_declarations(&ast, &[target]);
        assert_eq!(decls.classes.len(), 1);
        assert!(!decls.classes[0].is_abstract, "mixed class should not be abstract");
        let method_names: Vec<&str> = decls.classes[0].methods.iter().map(|m| m.name.as_str()).collect();
        // Non-pure virtual and regular methods are extracted.
        assert!(method_names.contains(&"virt_concrete"), "non-pure virtual should be extracted");
        assert!(method_names.contains(&"regular"), "regular method should be extracted");
        // Pure-virtual method in a mixed class is skipped conservatively.
        assert!(!method_names.contains(&"pure_one"), "pure-virtual in mixed class should be skipped");
        let reasons: Vec<&str> = decls.skipped.iter().map(|s| s.reason.as_str()).collect();
        assert!(reasons.contains(&"pure_virtual"));
    }
}
