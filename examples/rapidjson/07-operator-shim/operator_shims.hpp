// operator_shims.hpp — rapidjson-07-operator-shim
//
// ILLUSTRATION FILE — shows what cpp2rust-demo auto-generates in
//   .cpp2rust/<feature>/meta/operator_shims.hpp
//
// The full operator shim pipeline is fully automated:
//   1. cpp2rust-demo generates this file automatically (complete C++ shim bodies)
//   2. entry.rs receives an active import_lib! block (no commented-out starters)
//   3. The hicc::cpp! include and build.rs include paths are auto-configured
// No user action is required for standard operators.

// Auto-generated operator shims by cpp2rust-demo.
// Include this file in your hicc::cpp! block, then bind
// the functions below via #[cpp(func = "...")] in import_lib!.
#pragma once
#ifndef OPERATOR_SHIMS_HPP
#define OPERATOR_SHIMS_HPP

#include "value_with_ops.hpp"

/// Shim for `operator=` — copy assignment
static inline JsonValue& json_value_assign(JsonValue& self, const JsonValue& rhs) {
    return self = rhs;
}

/// Shim for `operator[]` — mutable member access by key
static inline JsonValue& json_value_get_at(JsonValue& self, const char* key) {
    return self[key];
}

/// Shim for `operator[]` — const member access by key
static inline const JsonValue& json_value_get_at_const(const JsonValue& self, const char* key) {
    return self[key];
}

/// Shim for `operator bool` — boolean conversion (explicit cast)
static inline bool json_value_to_bool(const JsonValue& self) {
    return static_cast<bool>(self);
}

/// Shim for `operator==` — equality comparison
static inline bool json_value_eq(const JsonValue& self, const JsonValue& rhs) {
    return self == rhs;
}

/// Shim for `operator!=` — inequality comparison
static inline bool json_value_ne(const JsonValue& self, const JsonValue& rhs) {
    return self != rhs;
}

#endif // OPERATOR_SHIMS_HPP
