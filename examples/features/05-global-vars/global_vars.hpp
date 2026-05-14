#pragma once
// Demonstrates extraction of global variables into Rust FFI.
//
// cpp2rust-demo extracts namespace-scoped global variables from C++ headers
// and generates `#[cpp(data = "...")]` bindings in `import_lib!`.
//
//  - Mutable globals  → `fn var_name() -> &'static mut T`
//  - Const globals    → `fn var_name() -> &'static T`
//
// The generated accessor returns a static reference, matching the global's
// lifetime.  The Rust name is snake_case (e.g. gMaxSize → g_max_size).

namespace metrics {

/// Mutable counter – changes at runtime.
extern int g_request_count;

/// Const threshold – set once at startup, then read-only.
extern const double g_max_latency_ms;

/// Plain global (no namespace) for comparison.
extern bool g_debug_enabled;

}  // namespace metrics
