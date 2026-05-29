#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct SharedData;

SharedData* shareddata_new(const char* name);
void shareddata_delete(SharedData* self);

int shareddata_use_count(SharedData* self);
const char* shareddata_getName(SharedData* self);

SharedData* shareddata_clone(SharedData* self);
void shareddata_reset(SharedData* self);

int shareddata_expired(SharedData* self);

struct Cache;

Cache* cache_new(void);
void cache_delete(Cache* self);

SharedData* cache_get(Cache* c, const char* name);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <string>
#include <unordered_map>

class SharedData {
    std::string name_;
public:
    int value;
    SharedData(const char* n);
    ~SharedData();
    int useCount() const;
    const char* getName() const;
    SharedData* clone() const;
    void reset();
};

class Cache {
    std::unordered_map<std::string, void*> data_;
public:
    Cache();
    ~Cache();
    SharedData* get(const char* name);
};

#endif
