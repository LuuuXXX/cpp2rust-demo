// entry.cpp — guided/01-std-string
#include "string_utils.hpp"

// Stub implementations — only declarations matter for AST extraction.
StringProcessor::StringProcessor(const char* /*prefix*/) {}
StringProcessor::~StringProcessor() {}

std::string StringProcessor::transform(const std::string& /*input*/) const { return {}; }
int StringProcessor::count_chars(const std::string& /*text*/, char /*c*/) const { return 0; }
std::string StringProcessor::join(const std::string& /*a*/, const std::string& /*b*/) const { return {}; }
const char* StringProcessor::prefix() const { return nullptr; }
