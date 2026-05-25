#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct DataFetcher;

struct DataFetcher* datafetcher_new(const char* name);
void datafetcher_delete(struct DataFetcher* self);

const char* datafetcher_getName(struct DataFetcher* self);
int datafetcher_getCacheCount(struct DataFetcher* self);

void datafetcher_refresh(struct DataFetcher* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
