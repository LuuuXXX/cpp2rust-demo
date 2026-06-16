#include "custom_deleter.h"

namespace custom_deleter_ns {

static int g_cleanup_count = 0;

void LoggingDeleter::operator()(std::string* p) const {
    ++g_cleanup_count;
    delete p;
}

int cleanup_count() { return g_cleanup_count; }

} // namespace custom_deleter_ns
