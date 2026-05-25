#include "mutable_member.h"
#include <iostream>
#include <cstring>

struct DataFetcher* datafetcher_new(const char* name) {
    return new DataFetcher(name);
}

void datafetcher_delete(struct DataFetcher* self) {
    delete self;
}

const char* datafetcher_getName(struct DataFetcher* self) {
    return self->getName();
}

int datafetcher_getCacheCount(struct DataFetcher* self) {
    return self->getCacheCount();
}

void datafetcher_refresh(struct DataFetcher* self) {
    self->refresh();
}

// DataFetcher class implementation
DataFetcher::DataFetcher(const char* n) : cache_count(0) {
    name = n;
}
DataFetcher::~DataFetcher() {}
const char* DataFetcher::getName() const { return name; }
int DataFetcher::getCacheCount() const { return cache_count; }
void DataFetcher::refresh() { cache_count++; }
