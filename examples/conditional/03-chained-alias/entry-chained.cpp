// entry-chained.cpp — conditional/03-chained-alias
//
// STEP B2: both aliases enabled → chained-alias transitive resolution.
//
// AliasRegistry::resolve_transitive() must track:
//   IntStore → Store<int>   (direct alias)
//   MyStore  → Store<int>   (resolved transitively via IntStore)
//
// Expected outputs:
//   types/mod.rs  : pub type IntStore = Store_i32;
//                   pub type MyStore  = Store_i32;
//   method/       : import_class! { class Store_i32 { ... } }

#include "store.hpp"

using IntStore = Store<int>;   // direct alias
using MyStore  = IntStore;     // chained alias

// Stub implementations (only declarations matter for AST extraction)
template<typename T> Store<T>::Store()  : entries_(nullptr), count_(0), capacity_(0) {}
template<typename T> Store<T>::~Store() {}
template<typename T> void Store<T>::put(const char* /*key*/, T /*v*/) {}
template<typename T> T    Store<T>::get(const char* /*key*/) const { return T{}; }
template<typename T> bool Store<T>::has(const char* /*key*/) const { return false; }
template<typename T> int  Store<T>::count()  const { return count_; }
template<typename T> void Store<T>::clear()        { count_ = 0; }
