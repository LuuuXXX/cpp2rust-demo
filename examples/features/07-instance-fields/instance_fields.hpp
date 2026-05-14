#pragma once
// Demonstrates extraction of public instance fields (non-static data members).
//
// cpp2rust-demo extracts `public` FieldDecl nodes from C++ classes and
// generates `#[cpp(field = "ClassName::field_name")]` read/write accessor
// bindings inside `import_class!`.
//
//  - Non-const field → getter `fn get_field(&self) -> &T`
//                      + setter `fn get_field_mut(&mut self) -> &mut T`
//  - Const field     → getter only `fn get_field(&self) -> &T`

struct Point {
    /// Mutable coordinate fields – getter + setter.
    double x;
    double y;

    /// Const label – getter only.
    const int id;

    Point(int id, double x, double y);

    /// Distance from origin.
    double length() const;
};
