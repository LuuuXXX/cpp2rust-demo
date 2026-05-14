#pragma once
// Demonstrates that `inline` functions are extracted exactly like
// non-inline free functions.  The `inline` keyword is a linker hint
// and has no effect on cpp2rust-demo's extraction logic.

namespace math {

/// Inline addition — inline keyword is transparent to the tool.
inline int add(int a, int b) { return a + b; }

/// Inline multiplication.
inline double mul(double x, double y) { return x * y; }

/// Non-inline function for comparison.
int subtract(int a, int b);

/// Inline overloaded function.
inline int clamp(int v, int lo, int hi) { return v < lo ? lo : v > hi ? hi : v; }

/// Non-inline overload of clamp.
double clamp(double v, double lo, double hi);

}  // namespace math
