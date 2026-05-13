// entry.cpp — rapidjson-05-virtual-methods
#include "virtual_allocator.hpp"
#include <cstdlib>

void*  BaseAllocator::Malloc(size_t size)                            { return std::malloc(size); }
void*  BaseAllocator::Realloc(void* p, size_t, size_t n)            { return std::realloc(p, n); }
void   BaseAllocator::Free(void* p)                                  { std::free(p); }
bool   BaseAllocator::CanFree() const                                { return true; }
size_t BaseAllocator::align(size_t size, size_t alignment) const {
    return (size + alignment - 1) & ~(alignment - 1);
}
const bool BaseAllocator::kNeedFree = true;
