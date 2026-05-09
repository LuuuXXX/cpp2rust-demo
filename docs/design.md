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
| Build interception | LD_PRELOAD | `clang -ast-dump=json` |
| Input | Build command | C++ header files |
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
C++ header(s)
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
# 1. Generate FFI for one or more headers
cpp2rust-demo init --link mylib path/to/mylib.hpp path/to/extra.hpp

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
│   ├── headers.json         ← list of input headers + link name
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

When multiple C++ functions share the same name (overloads), the tool:

1. Keeps the first occurrence as the plain Rust name.
2. Appends `_2`, `_3`, … for subsequent overloads.

Example:

```cpp
// C++
void process(int);    → process
void process(double); → process_2
void process(char*);  → process_3
```

The naming strategy is implemented in `ast::extract_function` and can be
extended to support configurable strategies (e.g., name-by-parameter-types).

## Known Limitations

1. **Constructors/destructors**: Skipped. Use factory functions or
   `hicc::make_unique<T>()` to create C++ objects.
2. **Virtual / pure-virtual methods**: Detected but not specially handled yet.
   Add them manually using `hicc`'s `#[interface]` attribute.
3. **Templates**: Not supported (complex without full type instantiation).
4. **Operator overloads**: Not supported (hicc limitation; use `hicc::cpp!` wrappers).
5. **Multiple inheritance**: Not supported by hicc.
6. **Anonymous structs/unions**: Not handled.
7. **`std::string` and STL types**: Passed through by bare class name; the user
   should add `hicc-std` as a dependency and use the appropriate mapped types.
8. **Double pointers (`T**`)**: Not handled (falls back to raw type name).
