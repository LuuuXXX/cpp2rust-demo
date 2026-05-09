# hicc Crate Usage Guide

This document explains how `cpp2rust-demo` uses the
[hicc](https://crates.io/crates/hicc) crate to generate safe, ergonomic Rust
FFI for C++ libraries.

## What is hicc?

`hicc` (short for *Hybrid Interface C++ Crate*) is a Rust crate that provides:

- **`hicc::import_lib!`** – maps C++ free functions (and static methods) to Rust.
- **`hicc::import_class!`** – maps a C++ class to a Rust struct with methods.
- **`hicc::cpp!`** – embeds raw C++ code (includes, helper functions, etc.).
- **`hicc_build::Build`** – a build helper that reads Rust source files
  containing the above macros and auto-generates the C++ adapter code.

## Three Related Crates

| Crate | Purpose |
|-------|---------|
| `hicc` | Core runtime types and macros |
| `hicc-std` | Pre-built mappings for STL containers (`vector`, `string`, `map`, …) |
| `hicc-build` | Build-time code generation (used in `build.rs`) |

## Key Macros

### `hicc::import_lib!`

Used to import **free C++ functions** and **static class methods**:

```rust
hicc::import_lib! {
    // Link name matches the C++ library.
    #![link_name = "mylib"]

    // Forward-declare any C++ class types used as parameters/returns.
    class Widget;

    // Map a free function.
    #[cpp(func = "int mylib::add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    // Map an overloaded function (renamed in Rust).
    #[cpp(func = "void mylib::process(double)")]
    fn process_double(value: f64);

    // Map a static class method.
    #[cpp(func = "int Widget::instanceCount()")]
    fn widget_instance_count() -> i32;
}
```

### `hicc::import_class!`

Used to map a **C++ class** to a Rust struct with methods:

```rust
hicc::import_class! {
    #[cpp(class = "Widget")]   // full C++ class name (with namespace if needed)
    class Widget {
        // Instance method (non-const → &mut self)
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);

        // Const method (→ &self)
        #[cpp(method = "int getId() const")]
        fn get_id(&self) -> i32;
    }
}
```

> **Note**: Static methods are NOT declared here. Use `import_lib!` instead.

### `hicc::cpp!`

Embed raw C++ code (useful for includes and helper wrappers):

```rust
hicc::cpp! {
    #include <iostream>
    static void greet(const char* name) {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
}

hicc::import_lib! {
    #![link_name = "mylib"]
    #[cpp(func = "void greet(const char*)")]
    fn greet(name: *const i8);
}
```

## `hicc_build` in `build.rs`

The generated `build.rs` uses `hicc_build::Build` to:
1. Parse your Rust source for `import_lib!` / `import_class!` macros.
2. Generate the corresponding C++ adapter code.
3. Compile it and link it into the Rust binary.

```rust
// build.rs
fn main() {
    hicc_build::Build::new()
        .rust_file("src/merged_ffi.rs")  // the file containing hicc macros
        .compile("cpp2rust_adapter");     // name of the compiled adapter

    println!("cargo::rustc-link-lib=cpp2rust_adapter");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rustc-link-lib=mylib");  // your actual C++ library
}
```

## Type Mapping Rules

hicc automatically converts types between Rust and C++ according to these rules.

### Return Type Mapping

| C++ return type | Rust type |
|-----------------|-----------|
| `T` (value) | `T` |
| `T&&` | `T` |
| `std::unique_ptr<T>` | `T` |
| `const T&` | `hicc::ClassRef<'_, T>` |
| `T&` | `hicc::ClassRefMut<'_, T>` |
| `T*` | `hicc::ClassMutPtr<'_, T, 1>` |
| `const T*` | `hicc::ClassPtr<'_, T, 1>` |

### Parameter Type Mapping

| C++ param type | Rust type |
|----------------|-----------|
| `T` (value) | `T` |
| `const T&` | `&T` |
| `T&` | `&mut T` |
| `const T*` | `&hicc::ClassPtr<'_, T, 1>` |
| `T*` | `&hicc::ClassMutPtr<'_, T, 1>` |

> cpp2rust-demo generates the **Rust parameter types** from C++ types. hicc
> then applies the above mapping when generating the C++ adapter code.

## Using hicc-std for STL Types

If your C++ library uses STL types, add `hicc-std` as a dependency:

```toml
[dependencies]
hicc = "0.2.3"
hicc-std = "0.1"
```

Then declare the STL class alias in your FFI:

```rust
hicc::import_lib! {
    #![link_name = "mylib"]

    // Map std::string to hicc_std::string
    class MyString = hicc_std::string;

    #[cpp(func = "std::string mylib::getName()")]
    fn get_name() -> MyString;
}
```

## Handling C++ Exceptions

hicc can capture C++ exceptions via `hicc::Exception<T>`:

```rust
hicc::import_lib! {
    #![link_name = "mylib"]

    // Wrap the return value in Exception<T> to catch C++ exceptions.
    #[cpp(func = "int riskyOperation()")]
    fn risky_operation() -> hicc::Exception<i32>;
}

fn main() {
    match risky_operation().ok() {
        Some(v) => println!("success: {}", v),
        None    => println!("C++ exception thrown"),
    }
}
```

## Further Reading

- [hicc on crates.io](https://crates.io/crates/hicc)
- [hicc documentation](https://docs.rs/hicc)
- [hicc-std documentation](https://docs.rs/hicc-std)
- [hicc-build documentation](https://docs.rs/hicc-build)
