// entry.cpp — conditional/03-chained-alias
//
// Demonstrates two-level (chained) type alias resolution:
//   using IntStore = Store<int>;      // direct alias — Store<int> is unlocked
//   using MyStore  = IntStore;        // chained alias — MyStore is also unlocked
//                                       via AliasRegistry::resolve_transitive()
//
// STEP A (baseline): all aliases commented out → Store<T> is skipped
// STEP B (unlocked): uncomment either block below and re-run init

#include "store.hpp"

// ── [UNLOCK STEP B1] Direct alias only ───────────────────────────────────────
// using IntStore = Store<int>;
// ─────────────────────────────────────────────────────────────────────────────

// ── [UNLOCK STEP B2] Chained alias (IntStore → Store<int>, MyStore → IntStore)
// using IntStore = Store<int>;   // first level: direct alias
// using MyStore  = IntStore;     // second level: chained alias
// ─────────────────────────────────────────────────────────────────────────────

// Stub implementations (only declarations matter for AST extraction)
template<typename T> Store<T>::Store()  : entries_(nullptr), count_(0), capacity_(0) {}
template<typename T> Store<T>::~Store() {}
template<typename T> void Store<T>::put(const char* /*key*/, T /*v*/) {}
template<typename T> T    Store<T>::get(const char* /*key*/) const { return T{}; }
template<typename T> bool Store<T>::has(const char* /*key*/) const { return false; }
template<typename T> int  Store<T>::count()  const { return count_; }
template<typename T> void Store<T>::clear()        { count_ = 0; }
