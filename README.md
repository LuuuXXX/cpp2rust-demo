# cpp2rust-demo

A demonstration tool that generates Rust FFI bindings for C++ libraries using:

- **clang** AST JSON (`-ast-dump=json`) to extract C++ declarations, and
- **[hicc](https://crates.io/crates/hicc)** macros to express the FFI in ergonomic Rust.

The tool supports a workflow similar to [`c2rust-demo`](https://github.com/LuuuXXX/c2rust-demo)
but targets **C++** (classes, namespaces, method overloads) instead of plain C,
and uses `hicc` instead of `bindgen`.

---

## Why hicc Instead of bindgen?

| Feature | bindgen | hicc |
|---------|---------|------|
| C++ classes | Limited | Full support |
| Methods (const/static/virtual) | Not directly | Yes |
| Namespaces | Limited | Yes |
| STL types | Opaque pointers | `hicc-std` wrappers |
| C++ exceptions | Unsafe | `hicc::Exception<T>` |
| Ease of use | Moderate | High (declarative macros) |

---

## Prerequisites

- Rust toolchain (stable ≥ 1.82)
- `clang` (≥ 9, tested with clang 18)
- A C++ compiler for the target library

```bash
# Install clang (Ubuntu/Debian)
sudo apt install clang

# Install clang (macOS)
brew install llvm
```

---

## Installation

```bash
git clone https://github.com/LuuuXXX/cpp2rust-demo.git
cd cpp2rust-demo
cargo install --path .
```

Or run directly from the checkout:

```bash
cargo run -- --help
```

---

## Quick Start

### 1. Write (or locate) a C++ header

```cpp
// mylib.hpp
#pragma once
namespace mylib {
    int add(int a, int b);
    double scale(double x, double factor);
    void process(int value);
    void process(double value);   // overloaded
}
```

### 2. Run `init`

```bash
cpp2rust-demo init --link mylib mylib.hpp
```

This:
1. Runs `clang -ast-dump=json` on the header.
2. Parses the AST to extract declarations.
3. Generates a Rust project in `.cpp2rust/default/rust/`.

Output:

```
.cpp2rust/default/
├── ast/mylib.ast.json          ← raw clang AST (for debugging)
├── meta/headers.json           ← stored config
├── meta/init-interface-report.md
└── rust/
    ├── Cargo.toml              ← depends on hicc + hicc-build
    ├── build.rs
    └── src/
        ├── lib.rs
        └── ffi_mylib.rs        ← generated hicc FFI
```

### 3. Run `merge`

```bash
cpp2rust-demo merge
```

Consolidates all per-header `ffi_*.rs` files into a single `merged_ffi.rs` and
updates `build.rs` + `lib.rs` to reference it.

### 4. Review and use the generated code

```rust
// .cpp2rust/default/rust/src/merged_ffi.rs
hicc::import_lib! {
    #![link_name = "mylib"]

    #[cpp(func = "int mylib::add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    #[cpp(func = "double mylib::scale(double, double)")]
    fn scale(x: f64, factor: f64) -> f64;

    // Overloaded: first keeps original name, subsequent get numeric suffix.
    #[cpp(func = "void mylib::process(int)")]
    fn process(value: i32);

    #[cpp(func = "void mylib::process(double)")]
    fn process_2(value: f64);
}
```

---

## Command Reference

### `init`

```
cpp2rust-demo init [OPTIONS] <HEADER>...

Arguments:
  <HEADER>...  One or more C++ header files to process

Options:
  --feature <FEATURE>              Feature name [default: default]
  --link <LINK>                    Link library name (required)
  --extra-clang-args <ARGS>        Extra args forwarded to clang
                                   (e.g. "-std=c++17 -I./include")
  --clang <CLANG>                  clang binary [env: CPP2RUST_CLANG]
                                   [default: clang]
```

Example with extra flags:

```bash
cpp2rust-demo init \
  --feature myfeature \
  --link mylib \
  --extra-clang-args "-std=c++17 -I./include -DENABLE_FEATURE" \
  include/mylib/api.hpp include/mylib/types.hpp
```

### `merge`

```
cpp2rust-demo merge [OPTIONS]

Options:
  --feature <FEATURE>  Feature name (must match a previous init) [default: default]
```

---

## C++ Class Support

For a class like:

```cpp
class Widget {
public:
    void update(double x, double y);
    int getId() const;
    static int instanceCount();
};
```

The tool generates:

```rust
// Instance methods → import_class!
hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);

        #[cpp(method = "int getId() const")]
        fn get_id(&self) -> i32;
    }
}

// Static methods + forward declaration → import_lib!
hicc::import_lib! {
    #![link_name = "widget"]

    class Widget;

    #[cpp(func = "int Widget::instanceCount()")]
    fn widget_instance_count() -> i32;
}
```

---

## Overload Handling

C++ function overloads are resolved by appending a numeric suffix starting at
`_2` for the second occurrence:

| C++ declaration | Rust name |
|-----------------|-----------|
| `void process(int)` | `process` |
| `void process(double)` | `process_2` |
| `void process(const char*)` | `process_3` |

The naming strategy is implemented in `src/ast.rs` (`extract_function`) and can
be extended to support custom naming schemes.

---

## Examples

See the [`examples/`](examples/) directory:

| Example | Description |
|---------|-------------|
| [`examples/simple/`](examples/simple/) | Free functions in a namespace (with overloads) |
| [`examples/class/`](examples/class/) | C++ class with methods (const, static, non-const) |

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/design.md`](docs/design.md) | Architecture, data flow, IR, type mapping |
| [`docs/hicc-usage.md`](docs/hicc-usage.md) | How to use hicc macros and hicc-build |
| [`docs/clang-ast.md`](docs/clang-ast.md) | How clang AST JSON is parsed |

---

## Current Limitations

| Feature | Status |
|---------|--------|
| Free functions | ✅ |
| Namespaces | ✅ |
| Class instance methods | ✅ |
| `const` methods | ✅ |
| `static` methods | ✅ |
| Function overloads | ✅ (numeric suffix) |
| Private/protected members | ✅ (auto-skipped) |
| Constructors/destructors | ⚠️ Skipped (use factory functions) |
| Virtual / pure-virtual | ⚠️ Not specially handled |
| Templates | ❌ Not supported |
| Operator overloads | ❌ Not supported (use `hicc::cpp!` wrapper) |
| Multiple inheritance | ❌ Not supported by hicc |
| STL types | ⚠️ Bare class name only; add `hicc-std` manually |

---

## Testing

```bash
cargo test              # unit tests (27)
cargo test --test cli_tests  # integration tests (15)
```

---

## License

MIT