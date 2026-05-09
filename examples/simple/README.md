# Simple Example – Free Functions

This example demonstrates generating Rust FFI for a C++ header that contains
free functions (including overloaded ones) inside a namespace.

## Source

- `mylib.hpp` – C++ header with free functions in `namespace mylib`
- `mylib.cpp` – implementation (compile separately)

## Running the Example

From the repository root:

```bash
# Step 1: generate FFI
cpp2rust-demo init --link mylib -- clang -x c++ -fsyntax-only examples/simple/mylib.hpp

# Step 2: consolidate into a single file
cpp2rust-demo merge

# Step 3: inspect the generated output
ls .cpp2rust/default/rust/src/
cat .cpp2rust/default/rust/src/merged_ffi.rs
```

## Expected Generated FFI

After running the above commands you should see a `merged_ffi.rs` similar to:

```rust
hicc::import_lib! {
    #![link_name = "mylib"]

    #[cpp(func = "int mylib::add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    #[cpp(func = "double mylib::scale(double, double)")]
    fn scale(x: f64, factor: f64) -> f64;

    #[cpp(func = "int mylib::string_length(const char *)")]
    fn string_length(str: *const i8) -> i32;

    #[cpp(func = "int mylib::log_message(const char *, const char *)")]
    fn log_message(level: *const i8, msg: *const i8) -> i32;

    // Overloaded functions get a numeric suffix.
    #[cpp(func = "void mylib::process(int)")]
    fn process(value: i32);

    #[cpp(func = "void mylib::process(double)")]
    fn process_2(value: f64);

    #[cpp(func = "void mylib::process(const char *)")]
    fn process_3(value: *const i8);
}
```

## Compiling with the Generated Project

```bash
# Copy the generated project
cp -r .cpp2rust/default/rust/ mylib-ffi/
cd mylib-ffi/

# Compile the C++ library
clang++ -std=c++14 -c -fPIC ../../examples/simple/mylib.cpp -o mylib.o
ar rcs libmylib.a mylib.o
# OR: clang++ -shared -fPIC ../../examples/simple/mylib.cpp -o libmylib.so

# Build the Rust crate (needs the library in the search path)
LIBRARY_PATH=. cargo build
```

## Overload Naming Convention

By default, cpp2rust-demo resolves naming conflicts from C++ function overloads
by appending a numeric suffix starting from `_2`:

| C++ overload | Rust name |
|---|---|
| `void process(int)` | `process` |
| `void process(double)` | `process_2` |
| `void process(const char*)` | `process_3` |

The naming strategy is implemented in `src/ast.rs::extract_function` and can be
customised by modifying the overload resolution logic there.
