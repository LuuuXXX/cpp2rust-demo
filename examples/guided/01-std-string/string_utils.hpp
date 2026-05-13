#pragma once
// string_utils.hpp — guided/01-std-string
//
// C++ class using std::string for parameters and return values.
// Demonstrates the 🔧 guided workflow:
//   • Methods with std::string params/returns are SKIPPED by default
//     (hicc has no std::string ABI support).
//   • cpp2rust-demo generates an operator_shims.hpp starter with
//     C-string shim signatures; the user fills in the function bodies.

#include <string>

class StringProcessor {
public:
    explicit StringProcessor(const char* prefix);
    ~StringProcessor();

    // ── Methods with std::string ─────────────────────────────────────────
    // These will be SKIPPED; shim suggestions appear in operator_shims.hpp.

    /// Prepend the stored prefix to `input` and return the result.
    std::string transform(const std::string& input) const;

    /// Count occurrences of character `c` in `text`.
    int count_chars(const std::string& text, char c) const;

    /// Concatenate `a` and `b` with a separator.
    std::string join(const std::string& a, const std::string& b) const;

    // ── Method with only C-string / POD types ────────────────────────────
    // This will be EXTRACTED automatically (✅ 完全自动).

    /// Return the stored prefix as a C-string.
    const char* prefix() const;
};
