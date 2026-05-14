#pragma once
// Demonstrates extraction of static class data members.
//
// C++ static class data members are class-scoped globals.  cpp2rust-demo
// extracts them as `#[cpp(data = "ClassName::member")]` bindings in
// `import_lib!`, using the fully-qualified name.
//
//  - Mutable static member  → `fn member_name() -> &'static mut T`
//  - Const static member    → `fn member_name() -> &'static T`

class Counter {
public:
    Counter();

    /// Increment the counter.
    void increment();

    /// Get the current value.
    int get() const;

    /// Shared mutable counter (static non-const).
    static int instance_count;

    /// Shared limit (static const).
    static const int max_count;
};
