#include "mutable_member.h"
#include <cstring>

DataFetcher::DataFetcher(const char* n) : cache_count(0) {
    name = n;
}
DataFetcher::~DataFetcher() {}
const char* DataFetcher::getName() const { return name; }
int DataFetcher::getCacheCount() const { return cache_count; }
void DataFetcher::refresh() { cache_count++; }
