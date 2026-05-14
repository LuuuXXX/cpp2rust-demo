#pragma once
// A simple generic key-value store for the chained-alias demonstration.
// Without any alias, Store<T> is skipped (tool_conservative).
// A direct alias (using IntStore = Store<int>) unlocks extraction.
// A chained alias (using MyStore = IntStore) also unlocks extraction thanks
// to AliasRegistry::resolve_transitive().

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
