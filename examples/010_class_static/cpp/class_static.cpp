#include "class_static.h"

namespace class_static_ns {
int Counter::instance_count_ = 0;
int class_static_anchor() { return 10; }
}
