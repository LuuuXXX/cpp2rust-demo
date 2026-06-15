#pragma once

class DataFetcher {
    const char* name;
    mutable int cache_count;
    char cache_data[256];
public:
    DataFetcher(const char* n);
    ~DataFetcher();
    const char* getName() const;
    int getCacheCount() const;
    void refresh();
};
