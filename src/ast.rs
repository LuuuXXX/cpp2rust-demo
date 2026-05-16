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

/// Deserialise the `"value"` field of an `AstNode`, which clang emits either
/// as a JSON string (`"42"`) or as a raw integer (`42`).  Both forms are
/// normalised to `Option<String>`.
fn deserialize_value_field<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct ValueVisitor;

    impl<'de> Visitor<'de> for ValueVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a string, integer, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_any(ValueVisitor)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
    }

    deserializer.deserialize_option(ValueVisitor)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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
    /// Numeric or string value for constant-folded expressions.
    ///
    /// Clang emits a `"value"` field on `ConstantExpr` and `IntegerLiteral`
    /// nodes.  Used here to extract enum constant discriminant values.
    ///
    /// Clang may emit the value either as a JSON string (`"42"`) or as a raw
    /// JSON integer (`42`), depending on the node kind and clang version.
    /// The custom deserialiser below accepts both forms and normalises them to
    /// `Option<String>` so the rest of the extraction code is unaffected.
    #[serde(default, deserialize_with = "deserialize_value_field")]
    pub value: Option<String>,
    /// `"class"` when this `EnumDecl` is a scoped (`enum class`) enumeration.
    #[serde(rename = "scopedEnumTag")]
    pub scoped_enum_tag: Option<String>,
}

/// A direct base class entry as emitted in clang's `"bases"` array.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseSpecifier {
    pub access: String,
    #[serde(rename = "type")]
    pub type_info: TypeInfo,
    /// Whether this base is declared `virtual` (virtual inheritance / diamond).
    ///
    /// hicc does not support virtual inheritance; virtual bases are skipped
    /// and reported in the interface report.
    #[serde(rename = "isVirtual", default)]
    pub is_virtual: bool,
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
// Skip categorisation
// ---------------------------------------------------------------------------

/// Why a declaration was skipped during AST extraction.
///
/// Helps users distinguish "tool can be improved" skips from "hicc itself
/// cannot handle this" skips, as described in the refactoring plan (v2).
#[derive(Debug, Clone, Default, PartialEq)]
pub enum SkipCategory {
    /// The skip is a conservative tooling decision that could theoretically be
    /// removed (e.g. template specialisations without typedef aliases before
    /// the alias-registry feature was added, or mixed-class pure-virtual before
    /// the companion-interface feature).
    ToolConservative,
    /// hicc itself does not support this C++ construct (e.g. operator
    /// overloads, destructors, unbound template parameters).  A hand-written
    /// C++ shim is required.
    #[default]
    HiccLimitation,
}

// ---------------------------------------------------------------------------
// Alias registry
// ---------------------------------------------------------------------------

/// Registry of C++ `typedef`/`using` aliases built during the first AST pass.
///
/// Used to:
/// - Allow template-specialisation types through the type gate when they have
///   a user-defined alias.
/// - Determine the canonical Rust struct name for extracted template
///   specialisations.
#[derive(Debug, Default)]
pub struct AliasRegistry {
    /// Bare template name → all alias names for that template.
    /// e.g. `"GenericDocument"` → `["Document"]`
    ///
    /// A single template can have multiple aliases (e.g.
    /// `using A = T<int>; using B = T<double>;`), so we keep a Vec
    /// instead of a single String.
    template_to_alias: HashMap<String, Vec<String>>,
    /// Alias name → fully-qualified C++ type string.
    /// e.g. `"Document"` → `"rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator>"`
    alias_to_type: HashMap<String, String>,
    /// Full qualified C++ type → first alias name (reverse of `alias_to_type`).
    /// Used for precise per-specialisation lookups in
    /// `try_extract_template_spec`.
    type_to_alias: HashMap<String, String>,
}

impl AliasRegistry {
    /// Scan the whole AST and collect all `TypedefDecl` / `TypeAliasDecl`
    /// entries where the aliased type is a template specialisation (contains
    /// `<`).
    pub fn collect_from_ast(root: &AstNode) -> Self {
        let mut reg = AliasRegistry::default();
        collect_alias_nodes(root, &[], &mut reg);
        reg.resolve_transitive();
        reg
    }

    /// Resolve chains of aliases so that an alias of an alias of a template
    /// specialisation is treated the same as a direct alias.
    ///
    /// For example, given:
    /// ```cpp
    /// using A = SomeTemplate<int>;   // registered: A → "SomeTemplate<int>"
    /// using B = A;                   // initially:   B → "A"  (no '<')
    /// ```
    /// After this call `B` resolves to `"SomeTemplate<int>"` in
    /// `alias_to_type`, `type_to_alias` gains the reverse entry, and
    /// `template_to_alias` gains `"SomeTemplate" → ["A", "B"]`
    /// (`A` is the direct alias and is preferred; `B` is second-level).
    ///
    /// The algorithm iterates until no further resolutions are possible
    /// (fixed-point / transitive closure).
    fn resolve_transitive(&mut self) {
        loop {
            let mut changed = false;
            let keys: Vec<String> = self.alias_to_type.keys().cloned().collect();
            for key in &keys {
                let val = self.alias_to_type[key].clone();
                // If the stored value is already a template type, nothing to do.
                if val.contains('<') {
                    continue;
                }
                // Try to resolve through one more alias hop.
                if let Some(resolved) = self.alias_to_type.get(&val).cloned() {
                    if resolved.contains('<') {
                        // Found a template through the chain – update the entry
                        // to point directly to the template type.
                        *self.alias_to_type.get_mut(key).unwrap() = resolved.clone();
                        // Also register this alias in template_to_alias.
                        let bare = bare_template_name(&resolved);
                        if !bare.is_empty() {
                            let aliases =
                                self.template_to_alias.entry(bare.to_string()).or_default();
                            if !aliases.iter().any(|a| a == key) {
                                aliases.push(key.clone());
                            }
                        }
                        // Populate reverse map (first alias wins).
                        self.type_to_alias
                            .entry(resolved.clone())
                            .or_insert_with(|| key.clone());
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// True if `name` is a typedef/using alias whose underlying type (possibly
    /// reached through a transitive chain after [`resolve_transitive`]) is a
    /// template specialisation.
    pub fn is_alias_of_template(&self, name: &str) -> bool {
        self.alias_to_type
            .get(name)
            .is_some_and(|t| t.contains('<'))
    }

    /// Return the first alias Rust name for a bare template class name, if any.
    ///
    /// e.g. `alias_for_template("GenericDocument")` → `Some("Document")`
    ///
    /// When the same template has multiple aliases (e.g. two `using`
    /// declarations for different specialisations), the first registered alias
    /// is returned.  Use [`alias_for_type`] for a precise per-specialisation
    /// lookup.
    pub fn alias_for_template(&self, bare_name: &str) -> Option<&str> {
        self.template_to_alias
            .get(bare_name)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Return the alias name for a specific fully-qualified C++ type, if any.
    ///
    /// Unlike [`alias_for_template`], this performs an exact match on the
    /// fully-qualified type string (after stripping a leading `class ` or
    /// `struct ` keyword) so each distinct template specialisation maps to
    /// its own alias.
    ///
    /// e.g. `alias_for_type("rapidjson::GenericDocument<rapidjson::UTF8<char>>")` → `Some("Document")`
    pub fn alias_for_type(&self, full_qual_type: &str) -> Option<&str> {
        let t = full_qual_type.trim();
        let t = t
            .strip_prefix("class ")
            .or_else(|| t.strip_prefix("struct "))
            .unwrap_or(t)
            .trim();
        self.type_to_alias.get(t).map(|s| s.as_str())
    }

    /// Return the full qualified C++ type for an alias name, if registered.
    ///
    /// e.g. `full_type_for_alias("Document")` → `Some("rapidjson::GenericDocument<...>")`
    pub fn full_type_for_alias(&self, alias: &str) -> Option<&str> {
        self.alias_to_type.get(alias).map(|s| s.as_str())
    }

    /// True if the given bare template name (part before `<`) has an alias.
    pub fn has_template_alias(&self, bare_name: &str) -> bool {
        self.template_to_alias.contains_key(bare_name)
    }

    /// Insert one alias entry.  For `template_to_alias`, all aliases are kept
    /// (first registration still wins for the reverse `type_to_alias` map).
    fn insert(&mut self, alias_name: &str, full_qual_type: &str) {
        // Only register when the aliased type is a template specialisation.
        if full_qual_type.contains('<') {
            let bare_template = bare_template_name(full_qual_type);
            if !bare_template.is_empty() {
                let aliases = self
                    .template_to_alias
                    .entry(bare_template.to_string())
                    .or_default();
                if !aliases.iter().any(|a| a == alias_name) {
                    aliases.push(alias_name.to_string());
                }
            }
            // First alias wins for the precise reverse lookup.
            self.type_to_alias
                .entry(full_qual_type.to_string())
                .or_insert_with(|| alias_name.to_string());
        }
        self.alias_to_type
            .entry(alias_name.to_string())
            .or_insert_with(|| full_qual_type.to_string());
    }
}

/// Depth-first scan that collects typedef/using aliases into `reg`.
fn collect_alias_nodes(node: &AstNode, namespace: &[String], reg: &mut AliasRegistry) {
    match node.kind.as_str() {
        "NamespaceDecl" => {
            if let Some(ref ns_name) = node.name {
                let mut ns = namespace.to_vec();
                ns.push(ns_name.clone());
                for child in node.inner.iter().flatten() {
                    collect_alias_nodes(child, &ns, reg);
                }
            } else {
                for child in node.inner.iter().flatten() {
                    collect_alias_nodes(child, namespace, reg);
                }
            }
        }
        "TypedefDecl" | "TypeAliasDecl" => {
            if let (Some(alias_name), Some(full_type)) = (
                node.name.as_deref(),
                node.type_info.as_ref().map(|t| t.qual_type.as_str()),
            ) {
                // Register with both the bare name and the namespace-qualified name.
                reg.insert(alias_name, full_type);
                let qualified = make_qualified(namespace, alias_name);
                if qualified != alias_name {
                    reg.insert(&qualified, full_type);
                }
            }
            for child in node.inner.iter().flatten() {
                collect_alias_nodes(child, namespace, reg);
            }
        }
        // Do not descend into class bodies.  Typedefs / using-aliases defined
        // at class scope (e.g. `typedef Alloc<U> other` inside an allocator
        // `rebind` struct) are implementation details and must NOT be
        // registered as top-level type aliases.  Doing so would cause names
        // like `other` to be mistakenly used as Rust struct names when
        // extracting template specialisations.
        "CXXRecordDecl"
        | "ClassTemplateDecl"
        | "ClassTemplateSpecializationDecl"
        | "ClassTemplatePartialSpecializationDecl" => {}
        _ => {
            for child in node.inner.iter().flatten() {
                collect_alias_nodes(child, namespace, reg);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Operator shim IR
// ---------------------------------------------------------------------------

/// A C++ operator overload that was skipped during extraction.
///
/// Carries enough information to generate a hand-written shim in
/// `operator_shims.hpp` and a corresponding stub Rust binding.
#[derive(Debug, Clone)]
pub struct OperatorShimIR {
    /// Class owning the operator, if it is a member.
    pub class_name: Option<String>,
    /// Fully-qualified C++ class name (with namespace).
    pub qualified_class: Option<String>,
    /// Operator token, e.g. `"operator[]"`, `"operator=="`.
    pub operator_name: String,
    /// C++ return type string as emitted by clang (best-effort; may be empty).
    pub return_cpp_type: String,
    /// Suggested Rust shim function name (snake_case).
    pub shim_name: String,
    /// Parameters of the operator (not including the implicit `this`).
    pub params: Vec<ParamIR>,
    /// Whether the operator method is `const`-qualified.
    pub is_const: bool,
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
    #[allow(dead_code)]
    pub is_virtual: bool,
    /// Whether this is a pure-virtual method.
    #[allow(dead_code)]
    pub is_pure: bool,
    /// Class name, if this is a method.
    pub class_name: Option<String>,
    /// Whether the last C++ parameter was `va_list` (C-style variadic).
    ///
    /// When `true`, the `va_list` parameter is dropped from `params` and an
    /// `unsafe fn` binding with a trailing `...` marker is generated instead.
    pub is_variadic: bool,
    /// Whether the method is rvalue-ref qualified (`&&`).
    ///
    /// When `true`, the Rust binding uses `self` (by value / consuming) instead of
    /// `&mut self`.  This mirrors the C++ `&&`-qualified method semantic where
    /// the call requires an rvalue object and conceptually consumes it.
    pub is_rvalue: bool,
}

/// A single function parameter.
#[derive(Debug, Clone)]
pub struct ParamIR {
    pub name: String,
    pub cpp_type: String,
    pub rust_type: String,
}

/// A public instance field (non-static data member) of a C++ class.
///
/// Extracted from `FieldDecl` nodes inside a `CXXRecordDecl`.
/// Generates `#[cpp(field = "...")]` read/write accessor bindings in `import_class!`.
#[derive(Debug, Clone)]
pub struct FieldIR {
    /// Bare C++ field name, e.g. `"count"`.
    pub name: String,
    /// Rust-idiomatic snake_case name derived from `name`.
    pub rust_name: String,
    /// Fully-qualified accessor form: `"ClassName::field_name"`.
    ///
    /// Used in `#[cpp(field = "...")]` to identify the field.
    pub qualified_name: String,
    /// C++ type string as emitted by clang (e.g. `"int"`, `"double"`).
    pub cpp_type: String,
    /// Mapped Rust type (e.g. `"i32"`, `"f64"`).
    pub rust_type: String,
    /// Whether the field is `const`-qualified (only a read accessor is emitted).
    pub is_const: bool,
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
    /// Rust-idiomatic snake_case name for the accessor function.
    ///
    /// Derived from `name` via [`to_snake_case`], keeping consistency with
    /// how `FunctionIR::rust_name` is produced for free functions.
    pub rust_name: String,
    /// Fully namespace-qualified name, e.g. `"myns::g_counter"`.
    pub qualified_name: String,
    /// C++ type string as emitted by clang.
    pub cpp_type: String,
    /// Mapped Rust type string.
    pub rust_type: String,
    /// Whether the variable is `const`-qualified.
    pub is_const: bool,
    /// Set when this is a `static` data member of a class rather than a
    /// namespace-scope global.  Contains the bare class name.
    pub class_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Enum IR
// ---------------------------------------------------------------------------

/// A single enumerator constant from a C++ `enum` or `enum class`.
#[derive(Debug, Clone)]
pub struct EnumVariantIR {
    /// The C++ enumerator name, e.g. `"RED"`.
    pub name: String,
    /// Explicit discriminant value, if the constant-folded value was available
    /// in the clang AST.  When absent, Rust uses the implicit sequential value.
    pub value: Option<i64>,
}

/// A C++ `enum` or `enum class` declaration.
///
/// Extracted into a Rust `#[repr(C)] pub enum` definition emitted in the
/// `types/` semantic module so that Rust callers can use the named variants
/// without raw integer casts.
#[derive(Debug, Clone)]
pub struct EnumIR {
    /// Bare C++ enum name, e.g. `"Color"`.
    pub name: String,
    /// Fully namespace-qualified name, e.g. `"myns::Color"`.
    pub qualified_name: String,
    /// `true` when the C++ declaration is `enum class` (scoped).
    pub is_class: bool,
    /// Ordered list of public enumerator constants.
    pub variants: Vec<EnumVariantIR>,
}

// ---------------------------------------------------------------------------
// Alias IR
// ---------------------------------------------------------------------------

/// A simple C++ `typedef` or `using` alias whose underlying type is a
/// supported primitive or known class type.
///
/// Template aliases (those whose underlying type contains `<`) are handled
/// separately by `AliasRegistry` and result in `ClassIR` template-
/// specialisation records, not `AliasIR` records.
#[derive(Debug, Clone)]
pub struct AliasIR {
    /// The alias name as declared in C++ (e.g. `"MyInt"`).
    pub name: String,
    /// Fully namespace-qualified alias name (e.g. `"myns::MyInt"`).
    #[allow(dead_code)]
    pub qualified_name: String,
    /// The C++ type being aliased (e.g. `"unsigned int"`).
    pub aliased_cpp_type: String,
    /// The corresponding Rust type (e.g. `"u32"`).
    pub aliased_rust_type: String,
}

/// A C++ class or struct declaration.
#[derive(Debug, Clone, Default)]
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
    /// `true` when this class was extracted from a `ClassTemplateSpecializationDecl`
    /// (i.e. a concrete instantiation of a C++ template).
    pub is_template_specialization: bool,
    /// For template specialisations: the typedef/using alias used as the Rust
    /// struct identifier (e.g. `"Document"` for `GenericDocument<…>`).
    ///
    /// When `Some`, this name is used for `class <Name>` in `import_class!`
    /// while `qualified_name` is used for `#[cpp(class = "…")]`.
    pub canonical_name: Option<String>,
    /// Pure-virtual methods of a *mixed* class (has both concrete and
    /// pure-virtual public methods).
    ///
    /// These drive generation of a companion `#[interface]` trait named
    /// `{ClassName}Interface` that the concrete class inherits from.
    pub pure_virtual_methods: Vec<FunctionIR>,
    /// `true` when this class has pure-virtual methods alongside concrete
    /// methods (distinct from `is_abstract`, which means *all* methods are
    /// pure-virtual).
    pub has_pure_virtual: bool,
    /// Names of base classes declared with `virtual` keyword that were skipped
    /// because hicc does not support virtual inheritance.
    ///
    /// Populated for diagnostic / reporting purposes; these bases are **not**
    /// emitted in `import_class!`.
    pub skipped_virtual_bases: Vec<String>,
    /// Public non-static instance fields (data members) extracted from this class.
    ///
    /// Each field generates a pair of `#[cpp(field = "...")]` accessor functions
    /// in `import_class!`: a read accessor (`fn get_<name>(&self) -> &T`) and,
    /// for non-const fields, a mutable write accessor
    /// (`fn get_<name>_mut(&mut self) -> &mut T`).
    pub fields: Vec<FieldIR>,
}

/// A declaration that is intentionally skipped during extraction.
#[derive(Debug, Clone, Default)]
pub struct SkippedDecl {
    /// AST node kind, e.g. `CXXMethodDecl`.
    pub kind: String,
    /// Name or qualified name when available.
    pub name: String,
    /// Stable skip reason key.
    pub reason: String,
    /// Classification of why this was skipped.
    pub category: SkipCategory,
    /// For `ToolConservative` template skips: a suggested `using` alias
    /// declaration that, when added to the C++ header, would unlock automatic
    /// extraction on the next `init` run.
    ///
    /// Example: `"using MyDoc = rapidjson::GenericDocument<rapidjson::UTF8<char>>;"`
    pub suggested_alias: Option<String>,
    /// For functions skipped due to `std::string` or `std::function` parameters /
    /// return types: a ready-to-copy C++ shim function prototype that replaces
    /// the unsupported type with a hicc-compatible equivalent.
    ///
    /// For `std::string` this is a `static inline` wrapper that accepts / returns
    /// `const char*` instead.  For `std::function` this is a pure-virtual
    /// interface class skeleton with an `@make_proxy` usage hint.
    pub suggested_shim: Option<String>,
    /// When this skip was caused by an STL container parameter/return type
    /// (e.g. `"std::vector<Foo>"`, `"std::map<Key, Value>"`), this field holds
    /// the first such container type string encountered.
    ///
    /// Used to generate `hicc::RustAny<T>` / `hicc-std` type-mapping suggestions
    /// in the `types/` semantic module.
    pub stl_container_type: Option<String>,
}

/// All declarations extracted from a set of header files.
#[derive(Debug, Default)]
pub struct ExtractedDecls {
    pub functions: Vec<FunctionIR>,
    pub classes: Vec<ClassIR>,
    pub globals: Vec<GlobalVarIR>,
    /// C++ `enum` / `enum class` declarations.
    pub enums: Vec<EnumIR>,
    /// Simple `typedef` / `using` aliases for supported types.
    pub aliases: Vec<AliasIR>,
    pub skipped: Vec<SkippedDecl>,
    /// Operator overloads that were skipped.
    ///
    /// Used to generate `operator_shims.hpp` and stub Rust bindings.
    pub operator_shims: Vec<OperatorShimIR>,
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

    let v: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("parse clang AST JSON for {}: {}", header.display(), e))?;
    serde_json::from_value(v).map_err(|e| {
        anyhow!(
            "deserialize AstNode from clang AST JSON for {}: {}",
            header.display(),
            e
        )
    })
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

    // Second pass: collect typedef/using aliases for template types.
    // This lets us extract `ClassTemplateSpecializationDecl` nodes that have a
    // user-facing alias (e.g. `Document = GenericDocument<UTF8<char>, …>`).
    let alias_registry = AliasRegistry::collect_from_ast(ast_root);

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
        &alias_registry,
    );

    // Deduplicate extracted classes by Rust struct name (first occurrence wins).
    //
    // The same `ClassTemplateSpecializationDecl` can appear in TWO places in
    // the clang AST JSON:
    //   (a) as a child of the `ClassTemplateDecl` that wraps the template, and
    //   (b) as a standalone top-level node inside the enclosing namespace.
    // Both (a) and (b) are processed by `walk_node`, so without deduplication
    // the same class would appear twice in `result.classes`, producing two
    // `import_class!` blocks with the same struct name in the generated source
    // and ultimately the Rust `E0428` "defined multiple times" error.
    {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        result.classes.retain(|c| seen.insert(c.name.clone()));
    }

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
        // Also register ClassTemplateSpecializationDecl bare template names so
        // that `is_known_class_type` recognises them during type-gate checks.
        "ClassTemplateSpecializationDecl" => {
            if node.complete_definition.unwrap_or(false) {
                if let Some(tmpl_name) = node.name.as_deref() {
                    let qualified = make_qualified(namespace, tmpl_name);
                    map.entry(tmpl_name.to_string()).or_insert(qualified);
                }
            }
        }
        // Register enum names so that functions taking enum parameters pass the
        // type gate and are extracted rather than skipped.
        "EnumDecl" => {
            if let Some(enum_name) = node.name.as_deref().filter(|n| !n.is_empty()) {
                let qualified = make_qualified(namespace, enum_name);
                map.entry(enum_name.to_string()).or_insert(qualified);
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
    let (is_const, bare) = if let Some(rest) = core.strip_prefix("const ") {
        (true, rest.trim())
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
/// Returns `true` for namespace names that belong to the C++ standard library,
/// compiler runtime support, or other system-level implementation namespaces.
/// Declarations inside these namespaces are never useful as Rust FFI bindings
/// and filtering them at the namespace level prevents hundreds of unwanted
/// symbol extractions from preprocessed (all-headers-expanded) middleware files.
fn is_system_namespace(name: &str) -> bool {
    // Explicit well-known system namespaces.
    matches!(
        name,
        "std"
            | "__gnu_cxx"
            | "__cxx11"
            | "__1"
            | "__detail"
            | "__fs"
            | "__atomic_impl"
            | "posix"
            | "__pstl"
            | "__gnu_pbds"
            | "__exception_ptr"
            | "__cxxabiv1"
            | "abi"
            | "chrono"
            | "filesystem"
            | "experimental"
    ) || name.starts_with("__")
}

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

#[allow(clippy::too_many_arguments)]
fn walk_node(
    node: &AstNode,
    current_file: &mut String,
    targets: &[&Path],
    namespace: &[String],
    result: &mut ExtractedDecls,
    overload_counts: &mut HashMap<String, usize>,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
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
                // Skip well-known system/compiler namespaces to avoid extracting
                // stdlib and compiler-internal symbols from preprocessed headers.
                if is_system_namespace(ns_name) {
                    return;
                }
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
                        alias_registry,
                    );
                }
            }
        }

        "FunctionDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            // Skip compiler-internal names (double-underscore prefix).
            if node
                .name
                .as_deref()
                .map(|n| n.starts_with("__"))
                .unwrap_or(false)
            {
                return;
            }
            if is_operator_name(node.name.as_deref()) {
                collect_operator_shim(node, namespace, None, None, result);
                record_skipped(
                    result,
                    node,
                    namespace,
                    None,
                    "operator_overload",
                    SkipCategory::HiccLimitation,
                );
                return;
            }
            if let Some(ir) = extract_function(
                node,
                namespace,
                overload_counts,
                None,
                strategy,
                class_map,
                alias_registry,
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
            // Skip compiler-internal names (double-underscore prefix).
            if class_name.starts_with("__") {
                return;
            }
            let qualified_name = make_qualified(namespace, class_name);
            if let Some(class_ir) = extract_class_body(
                node,
                class_name,
                &qualified_name,
                false,
                None,
                namespace,
                current_file,
                result,
                strategy,
                class_map,
                alias_registry,
            ) {
                result.classes.push(class_ir);
            }
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
                    alias_registry,
                );
            }
        }

        // Template specialisation extraction.
        //
        // `ClassTemplateDecl` wraps a generic template.  We descend into it
        // looking for `ClassTemplateSpecializationDecl` children (concrete
        // instantiations) that have a typedef alias in the registry.
        // The uninstantiated template itself is skipped.
        "ClassTemplateDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            let mut extracted_any = false;
            for child in node.inner.iter().flatten() {
                if child.kind == "ClassTemplateSpecializationDecl"
                    && child.complete_definition.unwrap_or(false)
                {
                    if let Some(spec_class) = try_extract_template_spec(
                        child,
                        namespace,
                        current_file,
                        result,
                        strategy,
                        class_map,
                        alias_registry,
                    ) {
                        result.classes.push(spec_class);
                        extracted_any = true;
                    }
                }
            }
            if !extracted_any {
                // Collect concrete specialisation types from children so we can
                // suggest `using` aliases to the user.
                let suggested = collect_template_alias_suggestions(node, namespace);
                record_skipped_with_hint(
                    result,
                    node,
                    namespace,
                    None,
                    "template_decl",
                    // ToolConservative: adding a typedef alias unlocks extraction.
                    SkipCategory::ToolConservative,
                    suggested,
                );
            }
        }

        // Standalone `ClassTemplateSpecializationDecl` (explicit instantiation).
        "ClassTemplateSpecializationDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            if !node.complete_definition.unwrap_or(false) {
                return;
            }
            if let Some(spec_class) = try_extract_template_spec(
                node,
                namespace,
                current_file,
                result,
                strategy,
                class_map,
                alias_registry,
            ) {
                result.classes.push(spec_class);
            } else {
                let suggested = collect_template_alias_suggestions(node, namespace);
                record_skipped_with_hint(
                    result,
                    node,
                    namespace,
                    None,
                    "template_decl",
                    // ToolConservative: adding a typedef alias unlocks extraction.
                    SkipCategory::ToolConservative,
                    suggested,
                );
            }
        }

        // `FunctionTemplateDecl` — try to extract any concrete
        // (`FunctionDecl`) specialisations nested inside the template wrapper.
        // A concrete child has a `qualType` that does not mention
        // `"type-parameter-"` or `"dependent"`, meaning all template
        // parameters have been substituted with real types.
        "FunctionTemplateDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            let mut extracted_any = false;
            for child in node.inner.iter().flatten() {
                if child.kind != "FunctionDecl" {
                    continue;
                }
                // Skip the generic (un-instantiated) pattern: it still has
                // type-parameter tokens in its qualType.
                if let Some(ref ti) = child.type_info {
                    if ti.qual_type.contains("type-parameter-")
                        || ti.qual_type.contains("dependent")
                    {
                        continue;
                    }
                } else {
                    continue;
                }
                if is_operator_name(child.name.as_deref()) {
                    collect_operator_shim(child, namespace, None, None, result);
                    record_skipped(
                        result,
                        child,
                        namespace,
                        None,
                        "operator_overload",
                        SkipCategory::HiccLimitation,
                    );
                    continue;
                }
                if let Some(ir) = extract_function(
                    child,
                    namespace,
                    overload_counts,
                    None,
                    strategy,
                    class_map,
                    alias_registry,
                    &mut result.skipped,
                ) {
                    result.functions.push(ir);
                    extracted_any = true;
                }
            }
            if !extracted_any {
                record_skipped(
                    result,
                    node,
                    namespace,
                    None,
                    "template_decl",
                    // ToolConservative: add an explicit specialisation to unlock.
                    SkipCategory::ToolConservative,
                );
            }
        }

        // C++ enum / enum class extraction.
        "EnumDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            if let Some(enum_ir) = extract_enum(node, namespace) {
                result.enums.push(enum_ir);
            }
        }

        // Simple typedef / using aliases for supported types.
        "TypedefDecl" | "TypeAliasDecl" => {
            if !is_target(current_file, targets) {
                return;
            }
            let Some(alias_name) = node.name.as_deref() else {
                return;
            };
            let Some(ref type_info) = node.type_info else {
                return;
            };
            let cpp_type = type_info.qual_type.clone();
            // Template aliases are handled via AliasRegistry → ClassIR.
            if cpp_type.contains('<') {
                return;
            }
            // Only emit aliases whose underlying type we can map to Rust.
            if !is_supported_cpp_type(&cpp_type, class_map, alias_registry) {
                return;
            }
            let rust_type = cpp_to_rust_type_with_aliases(&cpp_type, alias_registry);
            let qualified_name = make_qualified(namespace, alias_name);
            result.aliases.push(AliasIR {
                name: alias_name.to_string(),
                qualified_name,
                aliased_cpp_type: cpp_type,
                aliased_rust_type: rust_type,
            });
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
            if let Some(global) = extract_global_var(node, namespace, class_map, alias_registry) {
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
                    alias_registry,
                );
            }
        }
    }
}

/// Try to extract a `ClassTemplateSpecializationDecl` as a `ClassIR`.
///
/// Returns `None` when the specialisation has no typedef alias in the registry
/// (we only extract specialisations that have a user-facing alias name).
fn try_extract_template_spec(
    node: &AstNode,
    namespace: &[String],
    current_file: &mut String,
    result: &mut ExtractedDecls,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
) -> Option<ClassIR> {
    let template_name = node.name.as_deref()?;

    // Prefer a precise match on the node's full specialisation type so that
    // two aliases for different specialisations of the same template
    // (e.g. `using A = T<int>; using B = T<double>;`) are handled correctly.
    // Fall back to the first registered alias for the bare template name when
    // no type_info is available.
    let alias = if let Some(ref ti) = node.type_info {
        alias_registry
            .alias_for_type(&ti.qual_type)
            .or_else(|| alias_registry.alias_for_template(template_name))
    } else {
        alias_registry.alias_for_template(template_name)
    }?;

    // Determine the full C++ type to use in `#[cpp(class = "…")]`.
    let qualified_name = alias_registry
        .full_type_for_alias(alias)
        .map(|s| s.to_string())
        .unwrap_or_else(|| make_qualified(namespace, template_name));
    let qualified_name = if qualified_name.is_empty() {
        make_qualified(namespace, template_name)
    } else {
        qualified_name
    };

    extract_class_body(
        node,
        alias,
        &qualified_name,
        true,
        Some(alias.to_string()),
        namespace,
        current_file,
        result,
        strategy,
        class_map,
        alias_registry,
    )
}

/// Core class-body extraction shared by `CXXRecordDecl` and
/// `ClassTemplateSpecializationDecl` paths.
///
/// * `class_name`    – the Rust struct name to use (alias for templates).
/// * `qualified_name`– the full C++ type for `#[cpp(class = "…")]`.
/// * `is_spec`       – true when called for a template specialisation.
/// * `canonical_name`– alias name for template specialisations.
#[allow(clippy::too_many_arguments)]
fn extract_class_body(
    node: &AstNode,
    class_name: &str,
    qualified_name: &str,
    is_spec: bool,
    canonical_name: Option<String>,
    namespace: &[String],
    current_file: &mut String,
    result: &mut ExtractedDecls,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
) -> Option<ClassIR> {
    let mut class_ir = ClassIR {
        name: class_name.to_string(),
        qualified_name: qualified_name.to_string(),
        is_template_specialization: is_spec,
        canonical_name,
        ..ClassIR::default()
    };

    // Extract public base classes.
    for base in &node.bases {
        if base.access == "public" {
            let raw = base.type_info.qual_type.trim();
            let bare = raw
                .strip_prefix("class ")
                .or_else(|| raw.strip_prefix("struct "))
                .unwrap_or(raw)
                .trim();
            if !bare.is_empty() {
                // Virtual bases are not supported by hicc; record them for
                // reporting but do not emit them in import_class!.
                if base.is_virtual {
                    class_ir.skipped_virtual_bases.push(bare.to_string());
                    continue;
                }
                // Prefer alias name if the base is a template with an alias.
                let resolved_base = {
                    let template_bare = bare_template_name(bare);
                    alias_registry
                        .alias_for_template(template_bare)
                        .map(|a| a.to_string())
                        .or_else(|| class_map.get(bare).cloned())
                        .unwrap_or_else(|| bare.to_string())
                };
                class_ir.bases.push(resolved_base);
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
                    record_skipped(
                        result,
                        child,
                        namespace,
                        Some(class_name),
                        "destructor",
                        SkipCategory::HiccLimitation,
                    );
                    continue;
                }
                if child.kind == "CXXConstructorDecl" {
                    if let Some(ctor) = extract_ctor(child, class_name, class_map, alias_registry) {
                        class_ir.ctors.push(ctor);
                    } else {
                        record_skipped(
                            result,
                            child,
                            namespace,
                            Some(class_name),
                            "constructor",
                            SkipCategory::HiccLimitation,
                        );
                    }
                    continue;
                }
                // Pure-virtual methods are held aside; we decide below.
                if child.is_pure.unwrap_or(false) {
                    pure_virtual_nodes.push(child.clone());
                    continue;
                }
                // Non-pure virtual and regular methods: both callable via hicc.
                if is_operator_name(child.name.as_deref()) {
                    collect_operator_shim(
                        child,
                        namespace,
                        Some(class_name),
                        Some(qualified_name),
                        result,
                    );
                    record_skipped(
                        result,
                        child,
                        namespace,
                        Some(class_name),
                        "operator_overload",
                        SkipCategory::HiccLimitation,
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
                    alias_registry,
                    &mut result.skipped,
                ) {
                    class_ir.methods.push(ir);
                }
            }
            // Static data members.
            "VarDecl" if child.storage_class.as_deref() == Some("static") => {
                if let Some(gv) = extract_static_member(
                    child,
                    class_name,
                    qualified_name,
                    class_map,
                    alias_registry,
                ) {
                    result.globals.push(gv);
                }
            }
            // Non-static instance fields.
            "FieldDecl" => {
                if let Some(field) =
                    extract_field(child, class_name, qualified_name, class_map, alias_registry)
                {
                    class_ir.fields.push(field);
                }
            }
            // Nested class / struct definitions.
            "CXXRecordDecl" if child.complete_definition.unwrap_or(false) => {
                if let Some(nested_name) = child.name.as_deref().filter(|n| !n.is_empty()) {
                    // Nested class lives in a "namespace" that includes the
                    // outer class name so qualified_name is correct.
                    let nested_ns: Vec<String> = namespace
                        .iter()
                        .cloned()
                        .chain(std::iter::once(class_name.to_string()))
                        .collect();
                    let nested_qualified = make_qualified(&nested_ns, nested_name);
                    if let Some(nested_class) = extract_class_body(
                        child,
                        nested_name,
                        &nested_qualified,
                        false,
                        None,
                        &nested_ns,
                        current_file,
                        result,
                        strategy,
                        class_map,
                        alias_registry,
                    ) {
                        result.classes.push(nested_class);
                    }
                }
            }
            _ => {}
        }
    }

    // Decide how to handle pure-virtual methods.
    //
    // - Fully abstract (no concrete methods): emit #[interface] with all PVMs.
    // - Mixed (concrete + pure-virtual): put PVMs in `pure_virtual_methods`
    //   for a companion `#[interface]` trait; main class remains concrete.
    // - Pure concrete (no PVMs): nothing extra to do.
    let has_concrete = !class_ir.methods.is_empty();
    let has_pvm = !pure_virtual_nodes.is_empty();

    if !has_concrete && has_pvm {
        // Fully abstract class → extract PVMs into `methods` (#[interface]).
        for pvm in &pure_virtual_nodes {
            if is_operator_name(pvm.name.as_deref()) {
                collect_operator_shim(
                    pvm,
                    namespace,
                    Some(class_name),
                    Some(qualified_name),
                    result,
                );
                record_skipped(
                    result,
                    pvm,
                    namespace,
                    Some(class_name),
                    "operator_overload",
                    SkipCategory::HiccLimitation,
                );
                continue;
            }
            if let Some(ir) = extract_function(
                pvm,
                namespace,
                &mut method_overloads,
                Some(class_name),
                strategy,
                class_map,
                alias_registry,
                &mut result.skipped,
            ) {
                class_ir.methods.push(ir);
            }
        }
        class_ir.is_abstract = true;
    } else if has_concrete && has_pvm {
        // Mixed class: extract pure-virtual methods into a companion interface.
        for pvm in &pure_virtual_nodes {
            if is_operator_name(pvm.name.as_deref()) {
                collect_operator_shim(
                    pvm,
                    namespace,
                    Some(class_name),
                    Some(qualified_name),
                    result,
                );
                record_skipped(
                    result,
                    pvm,
                    namespace,
                    Some(class_name),
                    "operator_overload",
                    SkipCategory::HiccLimitation,
                );
                continue;
            }
            if let Some(ir) = extract_function(
                pvm,
                namespace,
                &mut method_overloads,
                Some(class_name),
                strategy,
                class_map,
                alias_registry,
                &mut result.skipped,
            ) {
                class_ir.pure_virtual_methods.push(ir);
            }
        }
        class_ir.has_pure_virtual = true;
    } else {
        // Pure concrete class with no PVMs (normal case).
        // No action needed; the loop above already extracted all concrete methods.
    }

    // Sort ctors by ascending parameter count so that ctors[0] is always the
    // "simplest" constructor (used as the primary ctor in import_class!).
    class_ir.ctors.sort_by_key(|c| c.params.len());

    Some(class_ir)
}

/// Best-effort collection of an operator overload into `result.operator_shims`.
///
/// Called whenever an operator node is encountered and skipped.  We extract
/// as much type information as available (return type, param types) even when
/// the types are not fully supported, so that the generated shim header gives
/// the user something useful to start with.
fn collect_operator_shim(
    node: &AstNode,
    namespace: &[String],
    class_name: Option<&str>,
    qualified_class: Option<&str>,
    result: &mut ExtractedDecls,
) {
    let op_name = match node.name.as_deref() {
        Some(n) if n.starts_with("operator") => n,
        _ => return,
    };

    let (return_cpp_type, is_const) = if let Some(ref ti) = node.type_info {
        let qt = ti.qual_type.as_str();
        let is_c = qt.ends_with(") const");
        let ret = parse_fn_qual_type(qt).map(|(r, _)| r).unwrap_or_default();
        (ret, is_c)
    } else {
        (String::new(), false)
    };

    // Collect parameters (best-effort; ignore type-gate issues here).
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
                .unwrap_or("arg")
                .to_string();
            let cpp_type = p
                .type_info
                .as_ref()
                .map(|t| t.qual_type.clone())
                .unwrap_or_else(|| format!("/* unknown_type_{} */", i));
            let rust_type = cpp_to_rust_type(&cpp_type);
            ParamIR {
                name: pname,
                cpp_type,
                rust_type,
            }
        })
        .collect();

    let shim_name = operator_shim_fn_name(op_name, class_name);

    result.operator_shims.push(OperatorShimIR {
        class_name: class_name.map(|s| s.to_string()),
        qualified_class: qualified_class.map(|s| {
            // Use namespace-qualified class name if no qualified_class given.
            if s.is_empty() {
                class_name
                    .map(|c| make_qualified(namespace, c))
                    .unwrap_or_default()
            } else {
                s.to_string()
            }
        }),
        operator_name: op_name.to_string(),
        return_cpp_type,
        shim_name,
        params,
        is_const,
    });
}

/// Derive a snake_case shim function name for an operator.
///
/// Examples:
/// - `("operator[]", Some("Value"))` → `"value_get_at"`
/// - `("operator==", None)` → `"eq"`
fn operator_shim_fn_name(op: &str, class_name: Option<&str>) -> String {
    let bare_op = op.rsplit("::").next().unwrap_or(op);
    let suffix = match bare_op {
        "operator[]" => "get_at",
        "operator=" => "assign",
        "operator==" => "eq",
        "operator!=" => "ne",
        "operator<" => "lt",
        "operator<=" => "le",
        "operator>" => "gt",
        "operator>=" => "ge",
        "operator+" => "add",
        "operator-" => "sub",
        "operator*" => "mul",
        "operator/" => "div",
        "operator%" => "rem",
        "operator++" => "increment",
        "operator--" => "decrement",
        "operator()" => "call",
        "operator bool" => "to_bool",
        "operator int" => "to_int",
        "operator double" => "to_double",
        _ => "op",
    };
    if let Some(cls) = class_name {
        format!("{}_{}", to_snake_case(cls), suffix)
    } else {
        suffix.to_string()
    }
}

/// Extract a `FunctionIR` from a `FunctionDecl`, `CXXMethodDecl`, etc.
#[allow(clippy::too_many_arguments)]
fn extract_function(
    node: &AstNode,
    namespace: &[String],
    overload_counts: &mut HashMap<String, usize>,
    class_name: Option<&str>,
    strategy: &OverloadStrategy,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
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
            category: SkipCategory::HiccLimitation,
            suggested_alias: None,
            suggested_shim: None,
            stl_container_type: None,
        });
        return None;
    }

    if is_operator_name(Some(name)) {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "operator_overload".to_string(),
            category: SkipCategory::HiccLimitation,
            suggested_alias: None,
            suggested_shim: None,
            stl_container_type: None,
        });
        return None;
    }

    let Some(qual_type) = node.type_info.as_ref().map(|t| t.qual_type.as_str()) else {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
            category: SkipCategory::HiccLimitation,
            suggested_alias: None,
            suggested_shim: None,
            stl_container_type: None,
        });
        return None;
    };
    let Some((return_type, _)) = parse_fn_qual_type(qual_type) else {
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
            category: SkipCategory::HiccLimitation,
            suggested_alias: None,
            suggested_shim: None,
            stl_container_type: None,
        });
        return None;
    };
    if !is_supported_cpp_type(&return_type, class_map, alias_registry) {
        // Collect all ParmVarDecl types for shim generation even when we skip
        // due to the return type being unsupported.
        let all_param_types: Vec<(String, String)> = node
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
                    .map(|t| t.qual_type.clone())
                    .unwrap_or_default();
                (pname, cpp_type)
            })
            .collect();
        let shim = generate_unsupported_type_shim(name, class_name, &return_type, &all_param_types);
        let stl_container = find_stl_container_type(&return_type, &all_param_types);
        skipped.push(SkippedDecl {
            kind: node.kind.clone(),
            name: qualified_name.clone(),
            reason: "unsupported_type".to_string(),
            category: categorize_unsupported_type(&return_type),
            suggested_alias: None,
            suggested_shim: shim,
            stl_container_type: stl_container,
        });
        return None;
    }

    // Collect parameters from ParmVarDecl children.
    // Pre-collect so we know total count (needed to identify the last param).
    let parm_nodes: Vec<&AstNode> = node
        .inner
        .iter()
        .flatten()
        .filter(|c| c.kind == "ParmVarDecl")
        .collect();
    let parm_count = parm_nodes.len();

    let mut params: Vec<ParamIR> = Vec::new();
    let mut is_variadic = false;

    for (i, p) in parm_nodes.iter().enumerate() {
        let pname = p
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or(&format!("arg{}", i))
            .to_string();
        let Some(cpp_type) = p
            .type_info
            .as_ref()
            .map(|t| normalize_cpp_type(&t.qual_type))
        else {
            skipped.push(SkippedDecl {
                kind: node.kind.clone(),
                name: qualified_name.clone(),
                reason: "unsupported_type".to_string(),
                category: SkipCategory::HiccLimitation,
                suggested_alias: None,
                suggested_shim: None,
                stl_container_type: None,
            });
            return None;
        };

        // Special case: `va_list` (or its internal representation) as the last
        // parameter → variadic C-style function.  hicc supports this; we drop
        // the `va_list` param and generate an `unsafe fn` with `...` instead.
        if is_va_list_type(&cpp_type) && i == parm_count - 1 {
            is_variadic = true;
            continue; // skip adding to params
        }

        if !is_supported_cpp_type(&cpp_type, class_map, alias_registry) {
            // Collect all param types (including this one) for shim generation.
            let all_param_types: Vec<(String, String)> = parm_nodes
                .iter()
                .enumerate()
                .map(|(j, q)| {
                    let qname = q
                        .name
                        .as_deref()
                        .filter(|n| !n.is_empty())
                        .unwrap_or(&format!("arg{}", j))
                        .to_string();
                    let qt = q
                        .type_info
                        .as_ref()
                        .map(|t| t.qual_type.clone())
                        .unwrap_or_default();
                    (qname, qt)
                })
                .collect();
            let shim =
                generate_unsupported_type_shim(name, class_name, &return_type, &all_param_types);
            let stl_container = find_stl_container_type(&return_type, &all_param_types);
            skipped.push(SkippedDecl {
                kind: node.kind.clone(),
                name: qualified_name.clone(),
                reason: "unsupported_type".to_string(),
                category: categorize_unsupported_type(&cpp_type),
                suggested_alias: None,
                suggested_shim: shim,
                stl_container_type: stl_container,
            });
            return None;
        }
        let rust_type = cpp_to_rust_type_with_aliases(&cpp_type, alias_registry);
        params.push(ParamIR {
            name: pname,
            cpp_type,
            rust_type,
        });
    }

    let is_const = qual_type.ends_with(") const") || qual_type.ends_with("() const");
    // Clang emits `"T () &&"` for rvalue-ref qualified methods.  We also
    // trim trailing whitespace defensively in case of minor formatting
    // variations across clang versions.
    let rval_qt = qual_type.trim_end();
    let is_rvalue = rval_qt.ends_with(") &&") || rval_qt.ends_with("() &&");
    let is_static = node.storage_class.as_deref() == Some("static");
    let is_virtual = node.is_virtual.unwrap_or(false);
    let is_pure = node.is_pure.unwrap_or(false);

    // Build the C++ signature for hicc attributes.
    let qualified_return = qualify_cpp_type(&return_type, class_map);
    let param_types: Vec<String> = params
        .iter()
        .map(|p| qualify_cpp_type(&p.cpp_type, class_map))
        .collect();
    let method_suffix = if is_const {
        " const"
    } else if is_rvalue {
        " &&"
    } else {
        ""
    };
    let cpp_signature = format!(
        "{} {}({}){}",
        qualified_return,
        qualified_name,
        param_types.join(", "),
        method_suffix
    );

    // Overload resolution via the configured strategy.
    let overload_key = qualified_name.clone();
    let count = overload_counts.entry(overload_key).or_insert(0);
    *count += 1;
    let rust_name = strategy.uniquify(&to_snake_case(name), *count);

    let rust_return_type = cpp_to_rust_type_with_aliases(&return_type, alias_registry);

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
        is_variadic,
        is_rvalue,
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
/// This is the alias-aware version used internally.  When a type is a
/// template specialisation that has a known alias (e.g. `Document`), the
/// alias name is returned instead of the bare template name.
pub(crate) fn cpp_to_rust_type_with_aliases(
    cpp_type: &str,
    alias_registry: &AliasRegistry,
) -> String {
    let t = cpp_type.trim();

    // Pointer chain.
    if t.contains('*') && !t.contains("(*)") {
        let parts: Vec<&str> = t.split('*').collect();
        let ptr_depth = parts.len().saturating_sub(1);
        if ptr_depth > 0 {
            let base_part = parts[0].trim();
            let base_const = has_top_level_const(base_part);
            let base = strip_top_level_const(base_part);
            let mut rust_type = if base == "void" {
                "core::ffi::c_void".to_string()
            } else {
                cpp_to_rust_type_with_aliases(base, alias_registry)
            };
            for level in 0..ptr_depth {
                // In Rust FFI, only the DATA's const-ness matters.  C++ "pointer-const"
                // qualifiers (the `const` between stars, e.g. `char *const *`) apply to
                // the pointer address itself and are dropped when translating to Rust — exactly
                // as tools like rust-bindgen do.  Only the base type's own const qualifier
                // (e.g. `const char *`) determines whether the innermost pointer is `*const`.
                let pointee_const = level == 0 && base_const;
                rust_type = if pointee_const {
                    format!("*const {}", rust_type)
                } else {
                    format!("*mut {}", rust_type)
                };
            }
            return rust_type;
        }
    }

    // Reference.
    if let Some(inner) = strip_trailing_ref(t) {
        let is_const = has_top_level_const(inner);
        let base = strip_top_level_const(inner);
        let rust_base = cpp_to_rust_type_with_aliases(base, alias_registry);
        return if is_const {
            format!("&{}", rust_base)
        } else {
            format!("&mut {}", rust_base)
        };
    }

    // Strip top-level `const`.
    if has_top_level_const(t) {
        let rest = strip_top_level_const(t);
        return cpp_to_rust_type_with_aliases(rest, alias_registry);
    }

    // Primitive mappings (delegate to the alias-free version for these).
    match t {
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
        | "uintptr_t" => cpp_to_rust_type(t),
        _ => {
            // For user-defined / template types: first check for a template alias.
            let bare = bare_class_name(t);
            if let Some(alias) = alias_registry.alias_for_template(&bare) {
                return alias.to_string();
            }
            // Then check for a simple (non-template) typedef alias and resolve it
            // to its Rust type.  Single-level only — recursive resolution would
            // overflow the stack on deeply-chained std typedefs (e.g. spdlog).
            if let Some(underlying) = alias_registry.full_type_for_alias(&bare) {
                if !underlying.contains('<') {
                    let u = strip_top_level_const(underlying.trim());
                    return cpp_to_rust_type(u);
                }
            }
            bare
        }
    }
}

/// Map a C++ type string to a Rust type string suitable for hicc FFI.
///
/// This handles the common primitive types.  Complex types (user-defined
/// classes, templates, etc.) are passed through as the bare class name so
/// hicc can handle them via its `class` declarations.
pub fn cpp_to_rust_type(cpp_type: &str) -> String {
    // Normalize first: strip compiler-extension qualifiers (__restrict etc.)
    // that have no Rust equivalent. We need to own the string for the
    // normalization case; borrow it otherwise.
    let normalized;
    let t = if cpp_type.contains("__restrict") {
        normalized = normalize_cpp_type(cpp_type);
        normalized.as_str()
    } else {
        cpp_type.trim()
    };

    // Pointer chain: supports single and multi-level pointers, e.g.
    // `int **`, `const char **`, `const T * const *`.
    if t.contains('*') && !t.contains("(*)") {
        let parts: Vec<&str> = t.split('*').collect();
        let ptr_depth = parts.len().saturating_sub(1);
        if ptr_depth > 0 {
            let base_part = parts[0].trim();
            let base_const = has_top_level_const(base_part);
            let base = strip_top_level_const(base_part);
            let mut rust_type = if base == "void" {
                "core::ffi::c_void".to_string()
            } else {
                cpp_to_rust_type(base)
            };
            for level in 0..ptr_depth {
                // In Rust FFI, only the DATA's const-ness matters.  C++ "pointer-const"
                // qualifiers between stars are dropped — only the base type's const qualifier
                // determines whether the innermost pointer is `*const`.
                let pointee_const = level == 0 && base_const;
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
    if let Some(rest) = trimmed.strip_suffix('&') {
        Some(rest.trim_end())
    } else {
        None
    }
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

/// Normalize a C++ type string by removing compiler-extension qualifiers
/// that have no equivalent in Rust or standard C++ (e.g. `__restrict`,
/// `__restrict__`).
///
/// These qualifiers appear in clang's `qual_type` output for pointer parameters
/// on some platforms (e.g. `char *const *__restrict`).  hicc-build and
/// the Rust type conversion both fail when they encounter them, so we strip
/// them before any further processing.
pub(crate) fn normalize_cpp_type(t: &str) -> String {
    // Handle the common forms emitted by clang:
    //   "char *const *__restrict"   -> "char *const *"   (after `*`)
    //   "__restrict char *"         -> "char *"           (leading)
    //   "char * __restrict"         -> "char *"           (trailing / mid)
    //
    // Replacement order: handle the `*__restrict[__]` form first so that the
    // `*` is preserved; then handle any bare `__restrict[__]` that remains.
    // This avoids leaving an orphaned `*` after the longer suffix is matched.
    let s = t
        .replace("*__restrict__", "*")
        .replace("*__restrict", "*")
        .replace("__restrict__", "")
        .replace("__restrict", "");
    // Collapse any whitespace runs introduced by the removal and trim edges.
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract the bare (no namespace, no template args) outer template class name.
///
/// The correct order is: strip template args first (split on first `<`),
/// *then* strip the namespace (rsplit on `::`).  The reverse order produces
/// wrong results for namespace-qualified types like
/// `rapidjson::GenericDocument<rapidjson::UTF8<char>>` where `rsplit("::")`
/// first yields `"GenericDocument<rapidjson::UTF8<char>>"` and the subsequent
/// `split('<')` still gives `"GenericDocument"` — but only by accident.  For
/// deeper nesting like `rapidjson::GenericDocument<rapidjson::UTF8<char>,
/// rapidjson::CrtAllocator>`, `rsplit("::")` yields `"CrtAllocator>"` which
/// then can't be stripped further and produces a wrong result.
///
/// Examples:
///   `"rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator>"` → `"GenericDocument"`
///   `"std::vector<int>"` → `"vector"`
///   `"GenericValue<UTF8<char>>"` → `"GenericValue"`
///   `"int"` → `"int"`
pub(crate) fn bare_template_name(full_qual_type: &str) -> &str {
    // Step 1: everything before the first '<' (isolates outer class + namespace).
    let before_angle = full_qual_type
        .split('<')
        .next()
        .unwrap_or(full_qual_type)
        .trim();
    // Step 2: last '::' segment strips the namespace qualifier.
    before_angle
        .rsplit("::")
        .next()
        .unwrap_or(before_angle)
        .trim()
}

fn is_supported_cpp_type(
    cpp_type: &str,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
) -> bool {
    let t = cpp_type.trim();
    if t.is_empty() {
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
        return is_supported_cpp_type(base, class_map, alias_registry);
    }

    if let Some(inner) = strip_trailing_ref(t) {
        let base = strip_top_level_const(inner);
        return is_supported_cpp_type(base, class_map, alias_registry);
    }

    let base = strip_top_level_const(t);

    // Allow template types whose bare name has a typedef alias.
    if base.contains('<') {
        let bare_template = bare_template_name(base);
        if alias_registry.has_template_alias(bare_template) {
            return true;
        }
        // Still reject other unsupported template constructs.
        return false;
    }

    if contains_unsupported_type_construct(base) {
        return false;
    }

    // Check for a simple (non-template) typedef alias: look up the underlying
    // type and validate it directly (single level only).  Recursive resolution
    // would overflow the stack on deeply-chained std typedefs (e.g. spdlog).
    if let Some(underlying) = alias_registry.full_type_for_alias(base) {
        if !underlying.contains('<') {
            let u = strip_top_level_const(underlying.trim());
            return is_primitive_cpp_type(u) || is_known_class_type(u, class_map);
        }
        // After transitive resolution, this alias directly names a template
        // specialisation – treat it as supported (the alias is the Rust name).
        if alias_registry.is_alias_of_template(base) {
            return true;
        }
    }

    is_primitive_cpp_type(base) || is_known_class_type(base, class_map)
}

/// Determine the `SkipCategory` for an unsupported type string.
///
/// Template types that might become supported with an alias are categorised
/// as `ToolConservative`; truly unsupported constructs are `HiccLimitation`.
/// Function-pointer types are also `ToolConservative` because they can be
/// wrapped via a pure-virtual interface class + `@make_proxy`.
fn categorize_unsupported_type(cpp_type: &str) -> SkipCategory {
    let t = cpp_type.trim();
    if t.contains('<') || t.contains("type-parameter-") || t.contains("dependent") {
        SkipCategory::ToolConservative
    } else if is_function_pointer_type(t) {
        // Function pointers can be replaced by a virtual interface wrapper.
        SkipCategory::ToolConservative
    } else {
        SkipCategory::HiccLimitation
    }
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
    category: SkipCategory,
) {
    record_skipped_with_hint(result, node, namespace, class_name, reason, category, None);
}

fn record_skipped_with_hint(
    result: &mut ExtractedDecls,
    node: &AstNode,
    namespace: &[String],
    class_name: Option<&str>,
    reason: &str,
    category: SkipCategory,
    suggested_alias: Option<String>,
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
        category,
        suggested_alias,
        suggested_shim: None,
        stl_container_type: None,
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

/// Build a suggested `using` alias declaration for a skipped template.
///
/// Scans the `node`'s children for concrete `ClassTemplateSpecializationDecl`
/// nodes and returns a multi-line string with one `using` suggestion per unique
/// concrete instantiation type, e.g.:
/// ```text
/// // Concrete instantiation(s) found – add an alias to your header:
/// using MyBox_int = Box<int>;
/// using MyBox_float = Box<float>;
/// ```
///
/// Returns `None` when no concrete instantiations are found in the children.
fn collect_template_alias_suggestions(node: &AstNode, namespace: &[String]) -> Option<String> {
    // The template name is the bare name (e.g. "Box", "GenericDocument").
    let template_name = node.name.as_deref().unwrap_or("Unknown");
    let qualified_template = make_qualified(namespace, template_name);

    // Collect unique full instantiation type strings from specialisation children.
    let mut seen: Vec<String> = Vec::new();
    let children = node.inner.iter().flatten();
    for child in children {
        if child.kind == "ClassTemplateSpecializationDecl"
            && child.complete_definition.unwrap_or(false)
        {
            if let Some(ref ti) = child.type_info {
                let qt = ti.qual_type.trim();
                // Strip leading "struct " / "class " that clang sometimes emits.
                let qt = qt
                    .strip_prefix("struct ")
                    .or_else(|| qt.strip_prefix("class "))
                    .unwrap_or(qt)
                    .trim();
                if !qt.is_empty() && !seen.contains(&qt.to_string()) {
                    seen.push(qt.to_string());
                }
            }
        }
    }

    // If no concrete instantiations were found, fall back to a generic placeholder.
    if seen.is_empty() {
        // Use the template name to produce a minimal (but still actionable) hint.
        let placeholder = format!(
            "// Add a using/typedef alias for `{}` to unlock extraction, e.g.:\n// using My{}_1 = {}</* args */>;",
            qualified_template, template_name, qualified_template
        );
        return Some(placeholder);
    }

    let mut lines = String::from(
        "// Concrete instantiation(s) found – add a `using` alias to your header to unlock extraction:\n",
    );
    for (i, qt) in seen.iter().enumerate() {
        // Use the same My{bare}_{n} pattern as the suggest-aliases subcommand output.
        let bare = bare_template_name(qt);
        let alias_name = format!("My{}_{}", bare, i + 1);
        lines.push_str(&format!("// using {} = {};\n", alias_name, qt));
    }
    Some(lines.trim_end().to_string())
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
    alias_registry: &AliasRegistry,
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
            let is_copy =
                qt == format!("const {} &", bare) || qt == format!("const {} &", class_name);
            // Move: `ClassName &&`
            let is_move = qt == format!("{} &&", bare) || qt == format!("{} &&", class_name);
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
        let cpp_type_str = p.type_info.as_ref().map(|t| t.qual_type.as_str())?;
        if !is_supported_cpp_type(cpp_type_str, class_map, alias_registry) {
            return None;
        }
        let rust_type = cpp_to_rust_type_with_aliases(cpp_type_str, alias_registry);
        params.push(ParamIR {
            name: pname,
            cpp_type: cpp_type_str.to_string(),
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
    alias_registry: &AliasRegistry,
) -> Option<GlobalVarIR> {
    let name = node.name.as_deref()?.to_string();
    // Skip anonymous variables.
    if name.is_empty() || name == "<anonymous>" {
        return None;
    }

    let cpp_type = node.type_info.as_ref().map(|t| t.qual_type.clone())?;

    // Skip unsupported types (templates, function pointers, etc.).
    if !is_supported_cpp_type(&cpp_type, class_map, alias_registry) {
        return None;
    }

    let is_const = has_top_level_const(&cpp_type);
    let rust_type = if is_const {
        format!(
            "&'static {}",
            cpp_to_rust_type_with_aliases(strip_top_level_const(&cpp_type), alias_registry)
        )
    } else {
        format!(
            "&'static mut {}",
            cpp_to_rust_type_with_aliases(&cpp_type, alias_registry)
        )
    };

    let rust_name = to_snake_case(&name);
    let qualified_name = make_qualified(namespace, &name);

    Some(GlobalVarIR {
        name,
        rust_name,
        qualified_name,
        cpp_type,
        rust_type,
        is_const,
        class_name: None,
    })
}

/// Extract a static data member `VarDecl` from a class body.
///
/// Returns a `GlobalVarIR` whose `qualified_name` is `"ClassName::member"`
/// so that the codegen emits `#[cpp(data = "ClassName::member")]`.
fn extract_static_member(
    node: &AstNode,
    class_name: &str,
    class_qualified: &str,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
) -> Option<GlobalVarIR> {
    let name = node.name.as_deref()?.to_string();
    if name.is_empty() || name == "<anonymous>" {
        return None;
    }

    let cpp_type = node.type_info.as_ref().map(|t| t.qual_type.clone())?;
    if !is_supported_cpp_type(&cpp_type, class_map, alias_registry) {
        return None;
    }

    let is_const = has_top_level_const(&cpp_type);
    let rust_type = if is_const {
        format!(
            "&'static {}",
            cpp_to_rust_type_with_aliases(strip_top_level_const(&cpp_type), alias_registry)
        )
    } else {
        format!(
            "&'static mut {}",
            cpp_to_rust_type_with_aliases(&cpp_type, alias_registry)
        )
    };

    // Prefix the rust_name with the snake_case class name to avoid collisions.
    let rust_name = format!("{}_{}", to_snake_case(class_name), to_snake_case(&name));
    // The data accessor must use the fully-qualified C++ name.
    let qualified_name = format!("{}::{}", class_qualified, name);

    Some(GlobalVarIR {
        name,
        rust_name,
        qualified_name,
        cpp_type,
        rust_type,
        is_const,
        class_name: Some(class_name.to_string()),
    })
}

/// Extract a `FieldIR` from a `FieldDecl` node inside a class body.
///
/// Returns `None` when the field's type is unsupported or the field is anonymous.
fn extract_field(
    node: &AstNode,
    _class_name: &str,
    class_qualified: &str,
    class_map: &HashMap<String, String>,
    alias_registry: &AliasRegistry,
) -> Option<FieldIR> {
    let name = node.name.as_deref()?.to_string();
    if name.is_empty() || name == "<anonymous>" {
        return None;
    }

    let cpp_type = node.type_info.as_ref().map(|t| t.qual_type.clone())?;
    if !is_supported_cpp_type(&cpp_type, class_map, alias_registry) {
        return None;
    }

    let is_const = has_top_level_const(&cpp_type);
    let base_cpp = strip_top_level_const(&cpp_type);
    let rust_type = cpp_to_rust_type_with_aliases(base_cpp, alias_registry);
    let rust_name = to_snake_case(&name);
    let qualified_name = format!("{}::{}", class_qualified, name);

    Some(FieldIR {
        name,
        rust_name,
        qualified_name,
        cpp_type,
        rust_type,
        is_const,
    })
}

// ---------------------------------------------------------------------------
// Shim-suggestion helpers (P2: std::string / std::function; P3: fn-pointer / va_list)
// ---------------------------------------------------------------------------

/// True when `t` contains `std::string` or any `basic_string` variant.
///
/// Used to identify `std::string` parameter / return types that cannot be
/// passed through hicc directly (ABI mismatch) but can be shimmed via a
/// wrapper that accepts / returns `const char*` instead.
///
/// Matches both the canonical form (`std::string`, `std::basic_string<…>`) and
/// the GCC C++11 ABI form (`std::__cxx11::basic_string<…>`).
fn is_std_string_type(t: &str) -> bool {
    t.contains("std::string") || t.contains("basic_string")
}

/// True when `t` is a `std::function<…>` type.
///
/// These are callable wrappers that hicc does not support directly.  The
/// recommended pattern is to replace them with a pure-virtual interface class
/// and use `@make_proxy` to implement the interface from Rust.
fn is_std_function_type(t: &str) -> bool {
    let bare = t.trim();
    bare.starts_with("std::function<") || bare.contains(" std::function<")
}

/// True when `t` is a function pointer type (e.g. `"int (*)(int, double)"`).
///
/// Clang encodes these with `(*)` in the `qualType` string.  Function pointers
/// cannot be passed through hicc directly but can be replaced by a pure-virtual
/// C++ interface class whose implementor is connected via `@make_proxy`.
pub(crate) fn is_function_pointer_type(t: &str) -> bool {
    t.contains("(*)")
}

/// True when `t` is `va_list` or its platform-specific internal form.
///
/// Used to detect C-style variadic functions whose last parameter is `va_list`.
/// hicc supports these; the `va_list` parameter is dropped from the Rust
/// binding and an `unsafe fn` with trailing `...` is generated instead.
pub(crate) fn is_va_list_type(t: &str) -> bool {
    let bare = t.trim();
    bare == "va_list"
        || bare == "__va_list_tag *"
        || bare == "__builtin_va_list"
        || bare == "__va_list_tag [1]"
        // GCC/Clang internal representations: exact known forms only.
        || bare == "struct __va_list_tag *"
        || bare == "struct __va_list_tag[1]"
}

/// True when `t` looks like a standard STL sequential/associative container type
/// that could store arbitrary Rust data via `hicc::RustAny`.
///
/// Recognises the most common standard containers from both the `std::` and
/// `std::__cxx11::` namespaces as emitted by clang.
///
/// Does **not** match `std::string` / `std::function` (handled separately) or
/// `std::array<T,N>` (fixed-size; no heap allocation, different pattern).
pub(crate) fn is_stl_container_type(t: &str) -> bool {
    let bare = t.trim();
    // Standard sequential and associative containers.
    const STL_CONTAINERS: &[&str] = &[
        "std::vector<",
        "std::list<",
        "std::deque<",
        "std::forward_list<",
        "std::set<",
        "std::multiset<",
        "std::map<",
        "std::multimap<",
        "std::unordered_set<",
        "std::unordered_multiset<",
        "std::unordered_map<",
        "std::unordered_multimap<",
        "std::queue<",
        "std::stack<",
        "std::priority_queue<",
    ];
    STL_CONTAINERS.iter().any(|prefix| bare.contains(prefix))
}

/// Find the first STL container type in the return type or parameter list of a
/// skipped function / method.
///
/// Returns `Some(container_type_string)` if any is found, otherwise `None`.
fn find_stl_container_type(return_type: &str, params: &[(String, String)]) -> Option<String> {
    if is_stl_container_type(return_type) {
        return Some(return_type.to_string());
    }
    params
        .iter()
        .find(|(_, t)| is_stl_container_type(t))
        .map(|(_, t)| t.clone())
}

/// Generate a `suggested_shim` string for a function that was skipped because
/// one of its types is `std::string`, `std::function`, or a function pointer.
///
/// Returns `None` when none of these types is involved
/// (i.e. the skip is due to some other unsupported type).
fn generate_unsupported_type_shim(
    fn_name: &str,
    class_name: Option<&str>,
    return_type: &str,
    params: &[(String, String)],
) -> Option<String> {
    let has_string =
        is_std_string_type(return_type) || params.iter().any(|(_, t)| is_std_string_type(t));
    let has_function =
        is_std_function_type(return_type) || params.iter().any(|(_, t)| is_std_function_type(t));
    let has_fn_ptr = is_function_pointer_type(return_type)
        || params.iter().any(|(_, t)| is_function_pointer_type(t));

    if !has_string && !has_function && !has_fn_ptr {
        return None;
    }

    let mut out = String::new();

    if has_string {
        // Generate a C++ shim that replaces std::string with const char*.
        let shim_ret = if is_std_string_type(return_type) {
            "const char*".to_string()
        } else {
            return_type.to_string()
        };
        let shim_params: Vec<String> = params
            .iter()
            .map(|(pname, ptype)| {
                if is_std_string_type(ptype) {
                    format!("const char* {}", pname)
                } else {
                    format!("{} {}", ptype, pname)
                }
            })
            .collect();
        // Determine class prefix for the qualified call.
        let call_prefix = if class_name.is_some() {
            format!("obj.{}(", fn_name)
        } else {
            format!("{}(", fn_name)
        };
        let call_args: Vec<String> = params
            .iter()
            .map(|(pname, ptype)| {
                if is_std_string_type(ptype) {
                    format!("std::string({})", pname)
                } else {
                    pname.clone()
                }
            })
            .collect();
        let ret_prefix = if is_std_string_type(return_type) {
            "return "
        } else {
            ""
        };
        let ret_suffix = if is_std_string_type(return_type) {
            ".c_str()"
        } else {
            ""
        };
        let class_self = class_name
            .map(|c| format!("{} &obj", c))
            .unwrap_or_default();
        let all_params = if shim_params.is_empty() {
            class_self.clone()
        } else if class_self.is_empty() {
            shim_params.join(", ")
        } else {
            format!("{}, {}", class_self, shim_params.join(", "))
        };
        let shim_name = format!("{}_shim", fn_name);
        out.push_str(&format!(
            "// std::string shim for `{fn_name}` — replace std::string args/return with const char*\n"
        ));
        out.push_str(&format!(
            "// static inline {shim_ret} {shim_name}({all_params}) {{\n"
        ));
        if is_std_string_type(return_type) {
            out.push_str(&format!(
                "//   static std::string _ret = {call_prefix}{});\n",
                call_args.join(", ")
            ));
            out.push_str(&format!("//   {ret_prefix}_ret{ret_suffix};\n// }}\n"));
        } else {
            out.push_str(&format!(
                "//   {ret_prefix}{call_prefix}{}){ret_suffix};\n// }}\n",
                call_args.join(", ")
            ));
        }
    }

    if has_function {
        // Generate a pure-virtual interface class suggestion.
        // Find the first std::function param type to extract its signature.
        let fn_type = if is_std_function_type(return_type) {
            return_type
        } else {
            params
                .iter()
                .find(|(_, t)| is_std_function_type(t))
                .map(|(_, t)| t.as_str())
                .unwrap_or("")
        };
        let interface_name = format!(
            "{}Callback",
            fn_name
                .split("::")
                .last()
                .unwrap_or(fn_name)
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>()
        );
        out.push_str(&format!(
            "// std::function detected in `{fn_name}` — suggest virtual interface + @make_proxy:\n"
        ));
        out.push_str(&format!("// struct {interface_name} {{\n"));
        out.push_str(&format!("//   // Underlying type: {fn_type}\n"));
        out.push_str("//   virtual /* return_type */ operator()(/* args */) = 0;\n");
        out.push_str(&format!("//   virtual ~{interface_name}() = default;\n"));
        out.push_str("// };\n");
        out.push_str("// Then pass `&proxy_instance` instead of the std::function.\n");
        out.push_str("// Use hicc @make_proxy to implement the interface from Rust.\n");
    }

    if has_fn_ptr {
        // Generate a pure-virtual interface class suggestion for function pointer parameters.
        // Find the first function-pointer param to derive the interface name.
        let (fp_param_name, fp_type) = if is_function_pointer_type(return_type) {
            (fn_name.to_string(), return_type.to_string())
        } else {
            params
                .iter()
                .find(|(_, t)| is_function_pointer_type(t))
                .map(|(n, t)| (n.clone(), t.clone()))
                .unwrap_or_default()
        };
        let base_name = fp_param_name
            .split("::")
            .last()
            .unwrap_or(fp_param_name.as_str());
        let interface_name = format!(
            "{}Handler",
            base_name
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>()
        );
        out.push_str(&format!(
            "// Function pointer `{fp_param_name}` in `{fn_name}` — suggest virtual interface + @make_proxy:\n"
        ));
        out.push_str(&format!("// struct {interface_name} {{\n"));
        out.push_str(&format!("//   // Underlying type: {fp_type}\n"));
        out.push_str("//   virtual /* return_type */ call(/* args */) = 0;\n");
        out.push_str(&format!("//   virtual ~{interface_name}() = default;\n"));
        out.push_str("// };\n");
        out.push_str(&format!(
            "// Replace `{fp_param_name}` parameter with `{interface_name} *` and forward the call.\n"
        ));
        out.push_str("// Use hicc @make_proxy to implement the interface from Rust.\n");
    }

    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Extract an `EnumIR` from an `EnumDecl` AST node.
///
/// Returns `None` for anonymous enums (no name).  Enumerator constant values
/// are obtained from `ConstantExpr` or `IntegerLiteral` children when present
/// in the clang JSON output.
fn extract_enum(node: &AstNode, namespace: &[String]) -> Option<EnumIR> {
    let name = node.name.as_deref().filter(|n| !n.is_empty())?.to_string();
    let qualified_name = make_qualified(namespace, &name);
    let is_class = node
        .scoped_enum_tag
        .as_deref()
        .map(|t| t == "class")
        .unwrap_or(false);

    let mut variants: Vec<EnumVariantIR> = Vec::new();
    for child in node.inner.iter().flatten() {
        if child.kind != "EnumConstantDecl" {
            continue;
        }
        let Some(variant_name) = child.name.as_deref() else {
            continue;
        };
        let value = find_enum_constant_value(child);
        variants.push(EnumVariantIR {
            name: variant_name.to_string(),
            value,
        });
    }

    Some(EnumIR {
        name,
        qualified_name,
        is_class,
        variants,
    })
}

/// Find the integer discriminant value of an `EnumConstantDecl` node.
///
/// Clang emits the folded value on a `ConstantExpr` child (or directly on an
/// `IntegerLiteral` child).  When no value node is found, returns `None` and
/// Rust will use the implicit sequential discriminant.
fn find_enum_constant_value(node: &AstNode) -> Option<i64> {
    // Direct `value` field on the node itself (some clang versions / modes).
    if let Some(ref v) = node.value {
        if let Ok(n) = v.parse::<i64>() {
            return Some(n);
        }
    }
    // Otherwise search first-level children for a ConstantExpr or IntegerLiteral.
    for child in node.inner.iter().flatten() {
        if child.kind == "ConstantExpr" || child.kind == "IntegerLiteral" {
            if let Some(ref v) = child.value {
                if let Ok(n) = v.parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
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
        // Pointer-const qualifiers (between stars) are dropped in Rust FFI — only the
        // base type's const-ness determines *const vs *mut (same as rust-bindgen).
        assert_eq!(cpp_to_rust_type("const char * const *"), "*mut *const i8");
        assert_eq!(cpp_to_rust_type("char *const *"), "*mut *mut i8");
        // GCC/Clang __restrict qualifiers must also be stripped first.
        assert_eq!(cpp_to_rust_type("char *const *__restrict"), "*mut *mut i8");
        assert_eq!(cpp_to_rust_type("char *__restrict"), "*mut i8");
        assert_eq!(cpp_to_rust_type("const char *__restrict__"), "*const i8");
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
        let reg = AliasRegistry::default();
        assert!(is_supported_cpp_type("int", &map, &reg));
        assert!(is_supported_cpp_type("const Vec2 *", &map, &reg));
        assert!(is_supported_cpp_type("geo::Vec2 &", &map, &reg));
        assert!(!is_supported_cpp_type("std::vector<int> *", &map, &reg));
        assert!(!is_supported_cpp_type("Vec2 (*)(int)", &map, &reg));
        assert!(!is_supported_cpp_type("Document *", &map, &reg));
    }

    #[test]
    fn test_is_supported_cpp_type_with_alias() {
        let map = HashMap::new();
        let mut reg = AliasRegistry::default();
        // Simulate: using Document = GenericDocument<UTF8<char>, CrtAllocator>;
        reg.insert(
            "Document",
            "rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator>",
        );
        // The template type itself should be allowed through now.
        assert!(is_supported_cpp_type(
            "rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator>",
            &map,
            &reg
        ));
        assert!(is_supported_cpp_type(
            "rapidjson::GenericDocument<rapidjson::UTF8<char>, rapidjson::CrtAllocator> &",
            &map,
            &reg
        ));
        // Non-aliased templates still rejected.
        assert!(!is_supported_cpp_type("std::vector<int>", &map, &reg));
    }

    #[test]
    fn test_unsupported_type_helpers() {
        let map = HashMap::from([("Widget".to_string(), "ns::Widget".to_string())]);
        let reg = AliasRegistry::default();
        assert!(contains_unsupported_type_construct("std::vector<int>"));
        assert!(contains_unsupported_type_construct("int (*)()"));
        assert!(!contains_unsupported_type_construct("const Widget *"));

        assert!(is_primitive_cpp_type("int"));
        assert!(is_primitive_cpp_type("uint64_t"));
        assert!(!is_primitive_cpp_type("std::string"));

        assert!(is_known_class_type("Widget", &map));
        assert!(is_known_class_type("ns::Widget", &map));
        assert!(!is_known_class_type("Document", &map));

        // Aliased template types pass the type gate.
        assert!(!is_supported_cpp_type("std::vector<int>", &map, &reg));
        assert!(is_supported_cpp_type("Widget", &map, &reg));
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
            loc: Some(loc.clone()),
            inner: Some(vec![
                AstNode {
                    kind: "ClassTemplateDecl".to_string(),
                    loc: Some(loc.clone()),
                    name: Some("Box".to_string()),
                    ..AstNode::default()
                },
                AstNode {
                    kind: "FunctionDecl".to_string(),
                    loc: Some(loc.clone()),
                    name: Some("operator+".to_string()),
                    type_info: Some(TypeInfo {
                        qual_type: "int (int, int)".to_string(),
                    }),
                    inner: Some(vec![]),
                    ..AstNode::default()
                },
                AstNode {
                    kind: "CXXRecordDecl".to_string(),
                    loc: Some(loc.clone()),
                    name: Some("Widget".to_string()),
                    complete_definition: Some(true),
                    tag_used: Some("class".to_string()),
                    inner: Some(vec![
                        AstNode {
                            kind: "AccessSpecDecl".to_string(),
                            loc: Some(loc.clone()),
                            access: Some("public".to_string()),
                            ..AstNode::default()
                        },
                        AstNode {
                            kind: "CXXConstructorDecl".to_string(),
                            loc: Some(loc.clone()),
                            name: Some("Widget".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "void ()".to_string(),
                            }),
                            inner: Some(vec![]),
                            ..AstNode::default()
                        },
                        AstNode {
                            kind: "CXXMethodDecl".to_string(),
                            loc: Some(loc.clone()),
                            name: Some("virt".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "int ()".to_string(),
                            }),
                            is_virtual: Some(true),
                            is_pure: Some(true),
                            inner: Some(vec![]),
                            ..AstNode::default()
                        },
                        AstNode {
                            kind: "CXXMethodDecl".to_string(),
                            loc: Some(loc),
                            name: Some("operator[]".to_string()),
                            type_info: Some(TypeInfo {
                                qual_type: "int (int) const".to_string(),
                            }),
                            is_virtual: Some(false),
                            is_pure: Some(false),
                            inner: Some(vec![]),
                            ..AstNode::default()
                        },
                    ]),
                    ..AstNode::default()
                },
            ]),
            ..AstNode::default()
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
        assert_eq!(
            decls.classes[0].ctors.len(),
            1,
            "Widget() should be extracted as a CtorIR"
        );
        let reasons: Vec<&str> = decls.skipped.iter().map(|s| s.reason.as_str()).collect();
        assert!(reasons.contains(&"template_decl"));
        assert!(reasons.contains(&"operator_overload"));
        // constructor is NOT in skipped – it was extracted as CtorIR.
        assert!(
            !reasons.contains(&"constructor"),
            "extracted ctors should not appear in skipped list"
        );
        // pure_virtual is NOT in skipped for fully-abstract classes (it was extracted).
        assert!(!reasons.contains(&"pure_virtual"));
    }

    // -----------------------------------------------------------------------
    // AliasRegistry unit tests (1:N mapping, precise type lookup, transitive)
    // -----------------------------------------------------------------------

    /// Basic single-alias insertion and all lookup paths.
    #[test]
    fn test_alias_registry_insert_and_lookup() {
        let mut reg = AliasRegistry::default();
        reg.insert("Doc", "rapidjson::GenericDoc<rapidjson::UTF8<char>>");

        // alias_for_template returns the (sole) alias for the bare name.
        assert_eq!(
            reg.alias_for_template("GenericDoc"),
            Some("Doc"),
            "alias_for_template should find 'Doc' for bare name 'GenericDoc'"
        );
        // alias_for_type matches the full qualified type.
        assert_eq!(
            reg.alias_for_type("rapidjson::GenericDoc<rapidjson::UTF8<char>>"),
            Some("Doc"),
            "alias_for_type should find 'Doc' for the full type"
        );
        // full_type_for_alias rounds back.
        assert_eq!(
            reg.full_type_for_alias("Doc"),
            Some("rapidjson::GenericDoc<rapidjson::UTF8<char>>")
        );
        // has_template_alias and is_alias_of_template helpers.
        assert!(reg.has_template_alias("GenericDoc"));
        assert!(reg.is_alias_of_template("Doc"));
        // Unknown names return None / false.
        assert!(reg.alias_for_template("Other").is_none());
        assert!(reg.alias_for_type("Other<int>").is_none());
    }

    /// Two aliases for different specialisations of the same template are both
    /// stored; `alias_for_type` picks the correct one per specialisation while
    /// `alias_for_template` returns the first registered alias as a fallback.
    #[test]
    fn test_alias_registry_multiple_aliases_same_template() {
        let mut reg = AliasRegistry::default();
        // using IntBox  = Box<int>;
        // using StrBox  = Box<std::string>;
        reg.insert("IntBox", "Box<int>");
        reg.insert("StrBox", "Box<std::string>");

        // Both aliases are registered under the same bare template name.
        let aliases = reg
            .template_to_alias
            .get("Box")
            .expect("Box must be present");
        assert!(
            aliases.contains(&"IntBox".to_string()),
            "IntBox should be in the alias list"
        );
        assert!(
            aliases.contains(&"StrBox".to_string()),
            "StrBox should be in the alias list"
        );

        // Precise per-type lookup returns the matching alias.
        assert_eq!(
            reg.alias_for_type("Box<int>"),
            Some("IntBox"),
            "alias_for_type should return IntBox for Box<int>"
        );
        assert_eq!(
            reg.alias_for_type("Box<std::string>"),
            Some("StrBox"),
            "alias_for_type should return StrBox for Box<std::string>"
        );

        // alias_for_template falls back to the first registered alias.
        assert_eq!(
            reg.alias_for_template("Box"),
            Some("IntBox"),
            "alias_for_template should return the first alias (IntBox)"
        );

        // Both aliases resolve correctly through full_type_for_alias.
        assert_eq!(reg.full_type_for_alias("IntBox"), Some("Box<int>"));
        assert_eq!(reg.full_type_for_alias("StrBox"), Some("Box<std::string>"));
    }

    /// `alias_for_type` must strip a leading `class ` or `struct ` keyword
    /// before matching, since clang sometimes emits those prefixes in
    /// `type_info.qual_type`.
    #[test]
    fn test_alias_registry_alias_for_type_strips_prefix() {
        let mut reg = AliasRegistry::default();
        reg.insert("MyAlias", "Tmpl<int>");

        // Exact match (no prefix).
        assert_eq!(reg.alias_for_type("Tmpl<int>"), Some("MyAlias"));
        // With 'class ' prefix.
        assert_eq!(reg.alias_for_type("class Tmpl<int>"), Some("MyAlias"));
        // With 'struct ' prefix.
        assert_eq!(reg.alias_for_type("struct Tmpl<int>"), Some("MyAlias"));
        // Whitespace around the type.
        assert_eq!(reg.alias_for_type("  Tmpl<int>  "), Some("MyAlias"));
    }

    /// Inserting the same alias name twice (idempotent) must not duplicate the
    /// entry in the `template_to_alias` Vec.
    #[test]
    fn test_alias_registry_duplicate_insert_noop() {
        let mut reg = AliasRegistry::default();
        reg.insert("Alias", "Tmpl<int>");
        reg.insert("Alias", "Tmpl<int>");

        let aliases = reg
            .template_to_alias
            .get("Tmpl")
            .expect("Tmpl must be present");
        assert_eq!(
            aliases.iter().filter(|a| a.as_str() == "Alias").count(),
            1,
            "Alias should appear exactly once even after duplicate insertion"
        );
    }

    /// Transitive alias chains (`using B = A; using A = Tmpl<int>;`) must
    /// resolve: B ends up in `alias_to_type`, the Vec in `template_to_alias`,
    /// and `type_to_alias` so that `alias_for_type` / `alias_for_template`
    /// both find either alias.
    #[test]
    fn test_alias_registry_transitive_1n() {
        let mut reg = AliasRegistry::default();
        // Direct alias: A → Tmpl<int>
        reg.insert("A", "Tmpl<int>");
        // Indirect alias: B → A (not yet a template type).
        // Simulate what collect_alias_nodes does: insert B with val "A" (no '<').
        reg.alias_to_type.insert("B".to_string(), "A".to_string());

        reg.resolve_transitive();

        // After transitive resolution B should point directly to Tmpl<int>.
        assert_eq!(
            reg.full_type_for_alias("B"),
            Some("Tmpl<int>"),
            "B should transitively resolve to Tmpl<int>"
        );
        // B should also be in the template_to_alias Vec for "Tmpl".
        let aliases = reg
            .template_to_alias
            .get("Tmpl")
            .expect("Tmpl must be present");
        assert!(
            aliases.contains(&"B".to_string()),
            "B should appear in the alias Vec after transitive resolution"
        );
        // alias_for_type should find at least one of A or B for the type.
        let found = reg.alias_for_type("Tmpl<int>");
        assert!(
            found == Some("A") || found == Some("B"),
            "alias_for_type should return A or B for Tmpl<int>, got {:?}",
            found
        );
    }

    /// `collect_alias_nodes` must NOT register typedefs that live inside a
    /// class body (e.g. the `typedef Alloc<U> other` found inside an
    /// allocator's `rebind` helper struct).  Registering them would cause
    /// generic names like `other` to be mistakenly used as Rust struct names
    /// when extracting template specialisations.
    #[test]
    fn test_collect_alias_nodes_skips_class_scope_typedefs() {
        // Simulate the clang AST for:
        //
        //   template<typename T>
        //   struct StdAllocator {
        //       template<typename U>
        //       struct rebind {
        //           typedef StdAllocator<U> other;  // ← must NOT be collected
        //       };
        //   };
        //   using MyAlloc = StdAllocator<int>;      // ← must be collected
        //
        // The outer ClassTemplateDecl for StdAllocator wraps a CXXRecordDecl
        // that itself contains another CXXRecordDecl (rebind) which owns the
        // problematic TypedefDecl.

        let other_typedef = AstNode {
            kind: "TypedefDecl".to_string(),
            name: Some("other".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "StdAllocator<U>".to_string(),
            }),
            ..AstNode::default()
        };
        let rebind_record = AstNode {
            kind: "CXXRecordDecl".to_string(),
            name: Some("rebind".to_string()),
            complete_definition: Some(true),
            inner: Some(vec![other_typedef]),
            ..AstNode::default()
        };
        let alloc_record = AstNode {
            kind: "CXXRecordDecl".to_string(),
            name: Some("StdAllocator".to_string()),
            complete_definition: Some(true),
            inner: Some(vec![rebind_record]),
            ..AstNode::default()
        };
        let alloc_tmpl = AstNode {
            kind: "ClassTemplateDecl".to_string(),
            name: Some("StdAllocator".to_string()),
            inner: Some(vec![alloc_record]),
            ..AstNode::default()
        };
        // Namespace-level `using MyAlloc = StdAllocator<int>` – should be collected.
        let myalloc_alias = AstNode {
            kind: "TypeAliasDecl".to_string(),
            name: Some("MyAlloc".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "StdAllocator<int>".to_string(),
            }),
            ..AstNode::default()
        };
        let root = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            inner: Some(vec![alloc_tmpl, myalloc_alias]),
            ..AstNode::default()
        };

        let reg = AliasRegistry::collect_from_ast(&root);

        // The namespace-level alias must be collected.
        assert_eq!(
            reg.alias_for_template("StdAllocator"),
            Some("MyAlloc"),
            "MyAlloc should be registered as an alias for StdAllocator"
        );

        // The class-scope typedef `other` must NOT be registered.
        assert!(
            reg.full_type_for_alias("other").is_none(),
            "`other` (class-scope typedef inside rebind) must not be registered in AliasRegistry"
        );
        assert!(
            !reg.is_alias_of_template("other"),
            "`other` must not appear as an alias of a template"
        );
    }

    /// End-to-end: when a `ClassTemplateDecl` contains two
    /// `ClassTemplateSpecializationDecl` children, each with a distinct
    /// `type_info.qual_type` that maps to its own alias, `extract_declarations`
    /// must produce **two** separate `ClassIR` entries, each using its own
    /// alias as the Rust struct name (via `canonical_name`).
    #[test]
    fn test_extract_two_specialisations_use_distinct_aliases() {
        let target = Path::new("/tmp/two_specs.cpp");
        let loc = Location {
            file: Some(target.display().to_string()),
            line: None,
            col: None,
            offset: None,
            spelling_loc: None,
            expansion_loc: None,
            included_from: None,
        };

        // Build an AST that looks like:
        //   using IntBox = Box<int>;
        //   using StrBox = Box<std::string>;
        //   template<typename T> class Box { public: T value(); };
        //   // specialisations
        //   class Box<int> (complete)
        //   class Box<std::string> (complete)
        let alias_int = AstNode {
            kind: "TypeAliasDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("IntBox".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "Box<int>".to_string(),
            }),
            ..AstNode::default()
        };
        let alias_str = AstNode {
            kind: "TypeAliasDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("StrBox".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "Box<std::string>".to_string(),
            }),
            ..AstNode::default()
        };

        let make_spec = |qt: &str| AstNode {
            kind: "ClassTemplateSpecializationDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("Box".to_string()),
            complete_definition: Some(true),
            tag_used: Some("class".to_string()),
            type_info: Some(TypeInfo {
                qual_type: qt.to_string(),
            }),
            inner: Some(vec![]),
            ..AstNode::default()
        };

        let template_decl = AstNode {
            kind: "ClassTemplateDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("Box".to_string()),
            inner: Some(vec![make_spec("Box<int>"), make_spec("Box<std::string>")]),
            ..AstNode::default()
        };

        let ast = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            loc: Some(loc.clone()),
            inner: Some(vec![alias_int, alias_str, template_decl]),
            ..AstNode::default()
        };

        let decls = extract_declarations(&ast, &[target]);

        // Exactly two classes should be extracted (one per specialisation).
        assert_eq!(
            decls.classes.len(),
            2,
            "expected two ClassIR entries, one per specialisation alias; got {}: {:?}",
            decls.classes.len(),
            decls
                .classes
                .iter()
                .map(|c| c.canonical_name.as_deref().unwrap_or(&c.name))
                .collect::<Vec<_>>()
        );

        // Collect the Rust struct names (canonical_name when set, else name).
        let rust_names: Vec<&str> = decls
            .classes
            .iter()
            .map(|c| c.canonical_name.as_deref().unwrap_or(c.name.as_str()))
            .collect();

        assert!(
            rust_names.contains(&"IntBox"),
            "IntBox should be extracted; got {:?}",
            rust_names
        );
        assert!(
            rust_names.contains(&"StrBox"),
            "StrBox should be extracted; got {:?}",
            rust_names
        );

        // Both are flagged as template specialisations.
        for class in &decls.classes {
            assert!(
                class.is_template_specialization,
                "class {} should have is_template_specialization = true",
                class.name
            );
        }
    }

    /// Verify that a class with MIXED concrete + pure-virtual methods:
    /// - extracts the concrete methods normally (including non-pure virtual),
    /// - moves the pure-virtual methods to `pure_virtual_methods` (companion interface),
    /// - sets `has_pure_virtual = true`, and
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
        //     virtual int pure_one() = 0;   ← pure-virtual in mixed class → companion interface
        //     int regular();                ← regular → extract
        let make_method = |name: &str, is_virtual: bool, is_pure: bool| AstNode {
            kind: "CXXMethodDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some(name.to_string()),
            type_info: Some(TypeInfo {
                qual_type: "int ()".to_string(),
            }),
            is_virtual: Some(is_virtual),
            is_pure: Some(is_pure),
            inner: Some(vec![]),
            ..AstNode::default()
        };
        let ast = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            loc: Some(loc.clone()),
            inner: Some(vec![AstNode {
                kind: "CXXRecordDecl".to_string(),
                loc: Some(loc.clone()),
                name: Some("Mixed".to_string()),
                complete_definition: Some(true),
                tag_used: Some("class".to_string()),
                inner: Some(vec![
                    AstNode {
                        kind: "AccessSpecDecl".to_string(),
                        loc: Some(loc.clone()),
                        access: Some("public".to_string()),
                        ..AstNode::default()
                    },
                    make_method("virt_concrete", true, false),
                    make_method("pure_one", true, true),
                    make_method("regular", false, false),
                ]),
                ..AstNode::default()
            }]),
            ..AstNode::default()
        };

        let decls = extract_declarations(&ast, &[target]);
        assert_eq!(decls.classes.len(), 1);
        assert!(
            !decls.classes[0].is_abstract,
            "mixed class should not be abstract"
        );
        assert!(
            decls.classes[0].has_pure_virtual,
            "mixed class should have has_pure_virtual = true"
        );
        let method_names: Vec<&str> = decls.classes[0]
            .methods
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        // Non-pure virtual and regular methods are extracted into `methods`.
        assert!(
            method_names.contains(&"virt_concrete"),
            "non-pure virtual should be extracted"
        );
        assert!(
            method_names.contains(&"regular"),
            "regular method should be extracted"
        );
        // Pure-virtual method in a mixed class goes to `pure_virtual_methods`.
        assert!(
            !method_names.contains(&"pure_one"),
            "pure-virtual should not be in methods"
        );
        let pv_names: Vec<&str> = decls.classes[0]
            .pure_virtual_methods
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        assert!(
            pv_names.contains(&"pure_one"),
            "pure-virtual should be in pure_virtual_methods"
        );
        // pure_virtual is NOT in the skipped list any more.
        let reasons: Vec<&str> = decls.skipped.iter().map(|s| s.reason.as_str()).collect();
        assert!(
            !reasons.contains(&"pure_virtual"),
            "pure-virtual in mixed class should be extracted, not skipped"
        );
    }

    /// Regression test: when the same `ClassTemplateSpecializationDecl` appears
    /// as BOTH a child of its `ClassTemplateDecl` AND as a standalone top-level
    /// node in the same namespace (which clang regularly emits for implicit
    /// instantiations), `extract_declarations` must produce only ONE `ClassIR`
    /// entry for that specialisation — not two.
    ///
    /// Without the deduplication pass in `extract_declarations_with_strategy`,
    /// both occurrences would be extracted, producing two `import_class!` blocks
    /// with the same Rust struct name in the generated source and ultimately the
    /// Rust `E0428` "defined multiple times" error after merging.
    #[test]
    fn test_extract_declarations_deduplicates_classes_by_name() {
        let target = Path::new("/tmp/dedup_classes.cpp");
        let loc = Location {
            file: Some(target.display().to_string()),
            line: None,
            col: None,
            offset: None,
            spelling_loc: None,
            expansion_loc: None,
            included_from: None,
        };

        // Alias: using MyBox = Box<int>
        let alias_node = AstNode {
            kind: "TypeAliasDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("MyBox".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "Box<int>".to_string(),
            }),
            ..AstNode::default()
        };

        // A `Box<int>` ClassTemplateSpecializationDecl
        let spec_node = AstNode {
            kind: "ClassTemplateSpecializationDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("Box".to_string()),
            complete_definition: Some(true),
            tag_used: Some("class".to_string()),
            type_info: Some(TypeInfo {
                qual_type: "Box<int>".to_string(),
            }),
            inner: Some(vec![]),
            ..AstNode::default()
        };

        // ClassTemplateDecl that wraps the same specialisation as a child.
        let tmpl_node = AstNode {
            kind: "ClassTemplateDecl".to_string(),
            loc: Some(loc.clone()),
            name: Some("Box".to_string()),
            inner: Some(vec![spec_node.clone()]),
            ..AstNode::default()
        };

        // The root has: alias, template-decl (child spec), standalone spec.
        // This mimics the real clang AST where specialisations appear in both
        // positions.
        let root = AstNode {
            kind: "TranslationUnitDecl".to_string(),
            inner: Some(vec![alias_node, tmpl_node, spec_node]),
            ..AstNode::default()
        };

        let decls = extract_declarations(&root, &[target]);

        assert_eq!(
            decls.classes.len(),
            1,
            "duplicate ClassTemplateSpecializationDecl nodes must produce only ONE ClassIR; \
             got {}: {:?}",
            decls.classes.len(),
            decls.classes.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
        assert_eq!(
            decls.classes[0].name.as_str(),
            "MyBox",
            "class name should be the alias 'MyBox'"
        );
    }
}
