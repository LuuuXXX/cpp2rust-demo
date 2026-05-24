# cpp2rust-demo

`cpp2rust-demo` captures C++ builds with `LD_PRELOAD`, dumps Clang AST JSON for each translation unit, and generates hicc-based Rust FFI scaffolding under `.cpp2rust/<feature>/`.

## Installation

```bash
cargo build --release
# binary: target/release/cpp2rust-demo
```

Requirements: Linux, `g++`/`clang++`, `make`, and `clang++` with JSON AST dump support.

### hicc dependency

The generated Rust project depends on [hicc](https://gitcode.com/xuanwu/hicc) — a safe C++/Rust FFI library that is **not** published to crates.io.  After running `cpp2rust-demo init`, update the `hicc` and `hicc-build` path entries in the generated `.cpp2rust/<feature>/rust/Cargo.toml` to point at your local clone of the hicc repository:

```toml
[dependencies]
hicc = { path = "/your/local/hicc/hicc" }

[build-dependencies]
hicc-build = { path = "/your/local/hicc/hicc-build" }
```

## Usage

```bash
# capture and generate per-TU modules
cpp2rust-demo init --feature demo -- sh -c "make clean && make"

# merge duplicate declarations into a second source tree
cpp2rust-demo merge --feature demo
```

Generated layout:

```text
.cpp2rust/<feature>/
├── ast/
├── meta/
└── rust/
    ├── Cargo.toml
    ├── build.rs
    └── src/
```

## Supported C++ features

| # | Feature | Generator strategy |
|---|---|---|
| 01 | basic types | primitive type mapping + global `import_lib!` |
| 02 | pointers/references | pointer/reference heuristics to Rust refs/pointers |
| 03 | classes basic | `import_class!` + constructor factories |
| 04 | inheritance | base list rendered as `class Derived: Base` |
| 05 | virtual polymorphism | pure virtual => `#[interface]` |
| 06 | operator overload | shim helpers emitted in `hicc::cpp!` |
| 07 | templates function | explicit instantiated `FunctionDecl` only |
| 08 | templates class | explicit/specialized record declarations |
| 09 | namespaces | qualified names preserved |
| 10 | STL containers | hint comments for `hicc_std` review |
| 11 | smart pointers | `unique_ptr`/`shared_ptr` mapping |
| 12 | move semantics | `&&` methods map to `self` |
| 13 | lambdas/functional | `std::function` -> `hicc::Function` |
| 14 | type casting | dynamic-cast helper hints |
| 15 | exceptions | throwing methods return `hicc::Exception<T>` |
| 16 | static members | static methods emitted in `import_lib!` |
| 17 | friend functions | friend declarations treated as globals |
| 18 | const correctness | `&self` vs `&mut self` |
| 19 | memory management | placement-new helper hints |
| 20 | template specialization | specialized records/functions preserved when explicit |

## Limitations

| Area | Limitation | Reason |
|---|---|---|
| Templates | deep metaprogramming is not expanded automatically | AST-driven scaffolding only emits explicit declarations |
| Containers | complex STL APIs may need manual `hicc_std` wrappers | safe lifetimes and iterators need domain review |
| RTTI | dynamic-cast helpers are hinted, not exhaustively synthesized | runtime intent is not always recoverable from declarations |
| Exceptions | throw detection is heuristic (`throw` in source range) | JSON AST does not directly expose high-level Rust policy |
| Operators | generated shims cover common operators only | overloaded syntax needs per-signature adaptation |
| Build systems | Linux/`LD_PRELOAD` only | hook depends on Unix preload semantics |

## Validation

```bash
cargo build
cargo test
```
