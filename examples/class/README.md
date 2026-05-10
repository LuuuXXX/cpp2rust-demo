# Class Example – C++ Class with Methods

This example demonstrates generating Rust FFI for a C++ header that contains a
class with public methods (including `const` and `static` methods).

## Source

- `widget.hpp` – C++ class declaration
- `widget.cpp` – implementation

## Running the Example

```bash
# Step 1: capture build + interactive header selection + generate FFI
#
# Use a separate feature to avoid mixing with other examples.
# In a terminal: an interactive multi-select prompt lets you choose which
# captured headers to include.  In CI / non-interactive mode all headers are
# selected automatically.
cpp2rust-demo init --feature widget --link widget \
  -- clang -x c++ -fsyntax-only examples/class/widget.hpp

# Step 2: consolidate
cpp2rust-demo merge --feature widget

# Step 3: review
cat .cpp2rust/widget/rust/src/merged_ffi.rs
```

After `init` you will also find:

- `.cpp2rust/widget/meta/captured_headers.list` – all headers seen by the hook
- `.cpp2rust/widget/meta/selected_headers.json` – headers chosen in the
  selection step (auto-selected when stdin is not a terminal)

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
