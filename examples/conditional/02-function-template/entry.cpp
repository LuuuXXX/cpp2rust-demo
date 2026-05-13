// entry.cpp — conditional/02-function-template
//
// STEP A (baseline): no explicit specializations → templates are skipped.
// STEP B (unlocked): add explicit instantiation declarations below,
//                    then re-run cpp2rust-demo init.

#include "algorithms.hpp"

// ── [UNLOCK] Explicit instantiations for int / double ─────────────────────
// template int    clamp<int>(int, int, int);
// template void   swap_values<int>(int&, int&);
// template int    max_of<int>(int, int);
// template double lerp<double>(double, double, double);
// ──────────────────────────────────────────────────────────────────────────

// Stub implementations
template<typename T>
T clamp(T val, T lo, T hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
}

template<typename T>
void swap_values(T& a, T& b) {
    T tmp = a; a = b; b = tmp;
}

template<typename T>
T max_of(T a, T b) { return a > b ? a : b; }

template<typename T>
T lerp(T a, T b, double t) {
    return static_cast<T>(a + t * (b - a));
}
