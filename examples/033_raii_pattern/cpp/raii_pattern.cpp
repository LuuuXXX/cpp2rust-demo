#include "raii_pattern.h"

namespace raii_pattern_ns {

static int g_active_count = 0;
static int g_rollback_count = 0;

int active_count() { return g_active_count; }
int rollback_count() { return g_rollback_count; }

Resource::Resource(const char* name) : name_(name ? name : "") {
    ++g_active_count;
}

Resource::~Resource() {
    --g_active_count;
}

Transaction::Transaction() : committed_(false) {}

Transaction::~Transaction() {
    if (!committed_) {
        ++g_rollback_count;
    }
}

} // namespace raii_pattern_ns
