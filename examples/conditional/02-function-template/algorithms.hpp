#pragma once
// algorithms.hpp — conditional/02-function-template
//
// Generic function templates WITHOUT explicit specializations.
// Demonstrates the ⚠️ conditional workflow:
//   • Without explicit instantiation → function templates are skipped.
//   • After adding  `template int clamp<int>(int, int, int);`  in
//     entry.cpp → cpp2rust-demo automatically extracts that specialization.

/// Clamp `val` to the range [lo, hi].
template<typename T>
T clamp(T val, T lo, T hi);

/// Swap two values in place.
template<typename T>
void swap_values(T& a, T& b);

/// Return the greater of two values.
template<typename T>
T max_of(T a, T b);

/// Linear interpolation: a + t*(b-a).
template<typename T>
T lerp(T a, T b, double t);
