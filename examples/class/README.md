# Class Example – C++ Class with Methods

This example demonstrates generating Rust FFI for a C++ class by compiling a
translation unit and extracting APIs from the captured preprocessed middleware.

## Source

- `widget.hpp` – C++ class declaration
- `widget.cpp` – implementation

## Running the Example

From the repository root:

```bash
# Step 1: generate FFI (use a separate feature to avoid mixing with other examples)
cpp2rust-demo init --feature widget --link widget -- clang -x c++ -fsyntax-only examples/class/widget.cpp

# Step 2: consolidate
cpp2rust-demo merge --feature widget

# Step 3: review
cat .cpp2rust/widget/rust/src/merged_ffi.rs
```

> **Note**: In an interactive terminal, step 1 will prompt you to select which
> captured middleware files to include. Press `Space` to toggle, `Enter` to confirm.
> In non-interactive environments (CI, pipes) all files are selected automatically.

## Expected Generated FFI

```rust
// Instance methods go into import_class!
hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);

        #[cpp(method = "int getId() const")]
        fn get_id(&self) -> i32;

        #[cpp(method = "bool isVisible() const")]
        fn is_visible(&self) -> bool;
    }
}

// Static methods and forward declarations go into import_lib!
hicc::import_lib! {
    #![link_name = "widget"]

    class Widget;

    #[cpp(func = "int Widget::instanceCount()")]
    fn widget_instance_count() -> i32;
}
```

## How hicc Handles Classes

- Instance methods → `import_class!` with `#[cpp(method = "...")]`
- Static methods → `import_lib!` with `#[cpp(func = "...")]`  
- The class is forward-declared in `import_lib!` with `class Widget;`

## Current Limitations

| Feature | Status |
|---------|--------|
| Public instance methods | ✅ Supported |
| `const` methods | ✅ Supported (→ `&self`) |
| `static` methods | ✅ Supported (→ free fn in `import_lib!`) |
| Constructors / destructors | ⚠️ Skipped – use factory functions |
| Private / protected members | ✅ Automatically skipped |
| Virtual / pure-virtual methods | ⚠️ Skipped for now; add manually |
| Inheritance | ❌ Not yet supported |
| Templates | ❌ Not yet supported |
| Operator overloads | ❌ Not yet supported (hicc limitation) |
