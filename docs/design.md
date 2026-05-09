# Design Document: cpp2rust-demo

## Overview

`cpp2rust-demo` is a tool that generates Rust FFI bindings for C++ libraries by:

1. Running `clang -ast-dump=json` on C++ header files to produce an AST JSON.
2. Parsing the AST JSON to extract C++ declarations (functions, classes, methods).
3. Generating Rust source code using [hicc](https://crates.io/crates/hicc) macros.

## Comparison with c2rust-demo

| Aspect | c2rust-demo | cpp2rust-demo |
|--------|-------------|---------------|
| Target language | C | C++ |
| FFI generator | bindgen | hicc |
| Build interception | LD_PRELOAD | LD_PRELOAD hook (primary) + `clang -ast-dump=json` |
| Input | Build command | Build command (`init -- <BUILD_CMD...>`) |
| Class support | Structs only | Full C++ classes |
| Namespace support | N/A | Yes |
| Overload handling | N/A | Numeric suffix (_2, _3, …) |

## Architecture

```
cpp2rust-demo
├── src/
│   ├── main.rs       – CLI (init / merge subcommands)
│   ├── error.rs      – Error helpers
│   ├── layout.rs     – .cpp2rust/<feature>/ directory management
│   ├── ast.rs        – clang AST JSON parsing + IR extraction
│   ├── codegen.rs    – hicc FFI code generation
│   └── merge.rs      – Merge command (consolidates per-header files)
```

## Data Flow

```
Real build command (`init -- ...`)
    │
    ▼  LD_PRELOAD hook capture
Captured header set
    │
    ▼  clang -ast-dump=json
AstNode (clang JSON tree)
    │
    ▼  ast::extract_declarations()
ExtractedDecls (FunctionIR + ClassIR)
    │
    ▼  codegen::render_ffi()
ffi_<header>.rs (hicc macros)
    │
    ▼  merge::merge_ffi_files()
merged_ffi.rs (consolidated)
    │
    ▼  hicc_build (build.rs)
C++ adapter code + Rust FFI
```

## User Workflow

```bash
# 1. Capture real build and generate per-header FFI
cpp2rust-demo init --link mylib -- make -j4

# 2. Consolidate into a single file
cpp2rust-demo merge

# 3. Use the generated project
cp -r .cpp2rust/default/rust/ mylib-ffi/
cd mylib-ffi/ && cargo build
```

## Directory Layout

After `init + merge`, the `.cpp2rust/` directory contains:

```
.cpp2rust/<feature>/
├── ast/
│   └── <header>.ast.json   ← raw clang AST JSON (for debugging)
├── meta/
│   ├── headers.json         ← capture-derived headers used for AST/codegen + link name
│   ├── build_cmd.txt        ← raw build command passed to init
│   └── init-interface-report.md   ← summary of extracted declarations
└── rust/                   ← generated Rust project
    ├── Cargo.toml
    ├── build.rs
    └── src/
        ├── lib.rs
        ├── ffi_<header>.rs  ← one per input header (after init)
        └── merged_ffi.rs    ← consolidated (after merge)
```

## Intermediate Representation

The tool uses two main IR types:

### `FunctionIR`

Represents a single C++ function or method:

```rust
pub struct FunctionIR {
    pub name: String,           // original C++ name
    pub rust_name: String,      // snake_case, uniquified for overloads
    pub return_type: String,    // C++ return type
    pub rust_return_type: String,
    pub params: Vec<ParamIR>,
    pub qualified_name: String, // fully-qualified (e.g. "mylib::add")
    pub cpp_signature: String,  // for #[cpp(func = "...")]
    pub is_const: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_pure: bool,
    pub class_name: Option<String>,
}
```

### `ClassIR`

Represents a C++ class/struct with its public methods:

```rust
pub struct ClassIR {
    pub name: String,
    pub qualified_name: String,
    pub methods: Vec<FunctionIR>,
}
```

## Type Mapping

| C++ type | Rust type |
|----------|-----------|
| `void` | `()` |
| `bool` | `bool` |
| `char` / `signed char` | `i8` |
| `unsigned char` | `u8` |
| `short` | `i16` |
| `unsigned short` | `u16` |
| `int` / `signed int` | `i32` |
| `unsigned int` | `u32` |
| `long` / `long int` | `i64` |
| `unsigned long` | `u64` |
| `long long` | `i64` |
| `unsigned long long` | `u64` |
| `float` | `f32` |
| `double` | `f64` |
| `size_t` | `usize` |
| `int32_t`, `int64_t`, etc. | `i32`, `i64`, etc. |
| `const char*` | `*const i8` |
| `char*` | `*mut i8` |
| `const void*` | `*const core::ffi::c_void` |
| `void*` | `*mut core::ffi::c_void` |
| `const T&` | `&T` |
| `T&` | `&mut T` |
| `const T*` | `*const T` |
| `T*` | `*mut T` |
| Other class types | bare class name (hicc handles) |

## Overload Handling

Overload naming is encapsulated in the `OverloadStrategy` enum (`src/ast.rs`):

```rust
pub enum OverloadStrategy {
    /// Append _2, _3, ... to the second and subsequent overloads (default).
    NumericSuffix,
}
```

When multiple C++ functions share the same name (overloads), the strategy:

1. Keeps the first occurrence as the plain Rust name.
2. Uses `OverloadStrategy::uniquify(base, count)` for disambiguation.

Example:

```cpp
// C++
void process(int);    → process
void process(double); → process_2
void process(char*);  → process_3
```

Adding a new naming strategy (e.g., type-based suffixes or user rename maps)
requires only a new variant in `OverloadStrategy` and a new `match` arm in
`uniquify`.  Callers use `extract_declarations_with_strategy` to select a
non-default strategy.

## How hicc::cpp! and include paths work

hicc-build compiles a C++ adapter file from your Rust source.  For that
adapter to call namespace-qualified C++ functions (e.g. `mylib::add`), it must
`#include` the header that declares the namespace.

The generated FFI files contain:

```rust
hicc::cpp! {
    #include "mylib.hpp"   // just the basename
}
```

The generated `build.rs` adds the header's parent directory to the compiler
include path:

```rust
let mut build = hicc_build::Build::new();
build.rust_file("src/merged_ffi.rs");
build.include("/absolute/path/to/header/dir");
build.compile("cpp2rust_adapter");
```

This is verified end-to-end by the `generated_project_passes_cargo_check`
integration test, which runs `cargo check` on the real generated output.

## Support Level Matrix

The table below distinguishes what the tool *extracts*, *generates*, and has
been *verified* to compile with hicc-build.

| Feature | Extracted | Generated | Verified |
|---------|-----------|-----------|---------|
| Free functions | ✅ | ✅ | ✅ |
| Namespaces | ✅ | ✅ | ✅ via `hicc::cpp!` |
| Class instance methods | ✅ | ✅ | ✅ |
| `const` methods | ✅ | ✅ | ✅ |
| `static` methods | ✅ | ✅ | ✅ |
| Function overloads | ✅ | ✅ | ✅ |
| Primitive `T*` / `const T*` | ✅ | ✅ raw ptr | ✅ |
| Primitive `T&` / `const T&` | ✅ | ✅ `&mut T`/`&T` | ✅ |
| Class `T*` / `const T*` params/return | ✅ | ✅ raw ptr | ✅ |
| Class `T&` / `const T&` params/return | ✅ | ✅ `&mut T`/`&T` | ✅ |
| Same-namespace class types (auto-qualified) | ✅ | ✅ | ✅ |
| Private/protected members | ✅ (skipped) | — | — |
| Virtual / pure-virtual detection | ✅ | ⚠️ not yet mapped | — |
| Constructors / destructors | ✅ (skipped) | — | — |
| Templates | ❌ | — | — |
| Operator overloads | ✅ detected | ❌ no mapping | — |
| Multiple inheritance | — | ❌ hicc limitation | — |
| STL types | ✅ bare name | ⚠️ bare name | — |
| Double pointers (`T**`) | ✅ bare string | ⚠️ falls back | — |

## Known Limitations

1. **Constructors/destructors**: Skipped. Use factory functions or
   `hicc::make_unique<T>()` to create C++ objects.
2. **Virtual / pure-virtual methods**: Detected but not yet mapped.
   Add them manually using hicc's `#[interface]` attribute.
3. **Templates**: Not supported (require full type instantiation analysis).
4. **Operator overloads**: Not supported by the code generator (use
   `hicc::cpp!` wrappers manually).
5. **Multiple inheritance**: Not supported by hicc.
6. **Anonymous structs/unions**: Not handled.
7. **STL types**: Passed through by bare class name; add `hicc-std` as a
   dependency and use the appropriate mapped types.
8. **Double pointers (`T**`)**: Fall back to the raw type name string.
9. **Absolute include paths**: The generated `build.rs` uses absolute paths
   for the header include directories.  When copying the generated project
   to another machine, update `build.include(...)` accordingly.
