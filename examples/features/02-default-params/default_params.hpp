#pragma once
// Demonstrates how cpp2rust-demo handles default parameter values.
//
// C++ default arguments are NOT preserved in the FFI layer — each
// overload (or the function itself) is emitted as a separate Rust binding.
// Tail parameters that have defaults are still included in the extracted
// signature unless the user explicitly uses `suggest-aliases` / shims.

namespace config {

/// Single parameter, no defaults.
int get_timeout();

/// Function with a default parameter: cpp2rust-demo ignores the default
/// value and emits the full signature.
void set_timeout(int ms, bool notify = false);

/// Multiple defaults — all parameters are still extracted.
double lerp(double a, double b, double t = 0.5);

/// Default parameter using a named constant expression.
void log(const char* msg, int level = 1);

}  // namespace config
