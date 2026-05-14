// entry-chained.cpp — conditional/03-chained-alias
//
// STEP B2: both aliases enabled → chained-alias transitive resolution.
//
// AliasRegistry::resolve_transitive() must track:
//   IntStore → Store<int>   (direct alias)
//   MyStore  → Store<int>   (resolved transitively via IntStore)
//
// The template class is defined INLINE here (not via #include) so that the
// ClassTemplateDecl appears in this translation unit — the target file.
// Without that, is_target() would fail on the included-header location and
// no class would be extracted.
//
// The free function declarations force clang to emit a concrete
// ClassTemplateSpecializationDecl for Store<int>.  The explicit
// instantiation `template class Store<int>` is also required so that the
// specialization has complete_definition=true and all methods are visible
// (without it, clang only emits a partial/incomplete specialization).
//
// Expected outputs:
//   types/mod.rs  : pub type MyStore = IntStore;
//   method/       : import_class! { class IntStore { fn put(...); ... } }
//   free/         : fn has_entry(...); fn count_entries(...);

// ── Template class definition (inline, not via header include) ─────────────
template<typename T>
class Store {
public:
    Store();
    ~Store();

    void        put(const char* key, T value);
    T           get(const char* key) const;
    bool        has(const char* key) const;
    int         count() const;
    void        clear();

private:
    struct Entry { const char* key; T value; };
    Entry* entries_;
    int    count_;
    int    capacity_;
};
// ───────────────────────────────────────────────────────────────────────────

using IntStore = Store<int>;   // direct alias
using MyStore  = IntStore;     // chained alias

// Explicit instantiation definition: forces clang to emit a complete
// ClassTemplateSpecializationDecl for Store<int> (complete_definition=true,
// all methods non-implicit).  Without this, the specialization has
// complete_definition=false and no methods are extracted.
template class Store<int>;

// Free function declarations that USE the aliased types.
// These force clang to instantiate Store<int> in the AST so that
// cpp2rust-demo can extract the template specialisation as class IntStore.
bool   has_entry(const IntStore& store, const char* key);
bool   has_entry_v2(const MyStore& store, const char* key);
int    count_entries(const IntStore& store);

// Stub implementations (only declarations matter for AST extraction)
template<typename T> Store<T>::Store()  : entries_(nullptr), count_(0), capacity_(0) {}
template<typename T> Store<T>::~Store() {}
template<typename T> void Store<T>::put(const char* /*key*/, T /*v*/) {}
template<typename T> T    Store<T>::get(const char* /*key*/) const { return T{}; }
template<typename T> bool Store<T>::has(const char* /*key*/) const { return false; }
template<typename T> int  Store<T>::count()  const { return count_; }
template<typename T> void Store<T>::clear()        { count_ = 0; }
