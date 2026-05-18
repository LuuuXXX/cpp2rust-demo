// value_with_ops.hpp — rapidjson-07-operator-shim
//
// A minimal JSON value class that models RapidJSON's GenericValue operator
// surface.  Used to demonstrate the operator_shims.hpp three-step workflow:
//   1. cpp2rust-demo auto-generates operator_shims.hpp starter + commented-out bindings in entry.rs
//   2. User confirms / adjusts the C++ shim implementations
//   3. User includes operator_shims.hpp in hicc::cpp! and activates bindings

#pragma once

/// Minimal JSON value, modelling the operator surface of rapidjson::GenericValue.
class JsonValue {
public:
    /// Construct a null value.
    explicit JsonValue(int type = 0);

    /// Copy constructor (skipped by cpp2rust-demo — copy ctor skip).
    JsonValue(const JsonValue& rhs);

    // Regular (non-operator) methods — these ARE extracted.
    int  GetType() const;
    bool IsNull()  const;
    bool IsInt()   const;
    bool IsString() const;
    int  GetInt()  const;
    const char* GetString() const;
    void SetInt(int v);
    void SetString(const char* s, unsigned length);

    // ---------------------------------------------------------------
    // Operators — ALL skipped by cpp2rust-demo, but shims auto-generated
    // ---------------------------------------------------------------

    /// Assignment.  Shim: json_value_assign(JsonValue&, const JsonValue&)
    JsonValue& operator=(const JsonValue& rhs);

    /// Object member access by key.  Shim: json_value_get_at(JsonValue&, const char*)
    JsonValue& operator[](const char* key);

    /// Const object member access.  Shim: json_value_get_at_const(const JsonValue&, const char*)
    const JsonValue& operator[](const char* key) const;

    /// Boolean conversion.  Shim: json_value_to_bool(const JsonValue&)
    explicit operator bool() const;

    /// Equality comparison.  Shim: json_value_eq(const JsonValue&, const JsonValue&)
    bool operator==(const JsonValue& rhs) const;

    /// Inequality comparison.  Shim: json_value_ne(const JsonValue&, const JsonValue&)
    bool operator!=(const JsonValue& rhs) const;
};
