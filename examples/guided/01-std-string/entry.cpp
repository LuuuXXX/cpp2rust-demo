// entry.cpp — guided/01-std-string
#include "string_utils.hpp"
#include <cstring>
#include <algorithm>

StringProcessor::StringProcessor(const char* prefix) : prefix_(prefix) {}
StringProcessor::~StringProcessor() {}

std::string StringProcessor::transform(const std::string& input) const {
    return prefix_ + input;
}

int StringProcessor::count_chars(const std::string& text, char c) const {
    return static_cast<int>(std::count(text.begin(), text.end(), c));
}

std::string StringProcessor::join(const std::string& a, const std::string& b) const {
    return a + "|" + b;
}

const char* StringProcessor::prefix() const { return prefix_.c_str(); }
