#include "namespace_nested.h"

namespace foo {

namespace baz {

DataProcessor::DataProcessor() : multiplier_(3) {}

} // namespace baz

const char* get_version() { return "1.0.0"; }

int get_build_number() { return 42; }

int namespace_nested_anchor() { return 0; }

} // namespace foo
