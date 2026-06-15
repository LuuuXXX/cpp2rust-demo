#include "functional_bind.h"
#include <iostream>
#include <functional>
#include <string>

// AdderImpl implementation
AdderImpl::AdderImpl(int base) : base_value(base) {}
AdderImpl::~AdderImpl() {}
int AdderImpl::add(int value) {
    return base_value + value;
}

// Adder implementation
Adder::Adder(int base_value) : impl(new AdderImpl(base_value)) {}
Adder::~Adder() { delete impl; }

// MultiplierImpl implementation
MultiplierImpl::MultiplierImpl(int f) : factor(f) {}
MultiplierImpl::~MultiplierImpl() {}
int MultiplierImpl::multiply(int value) {
    return factor * value;
}

// Multiplier implementation
Multiplier::Multiplier(int factor) : impl(new MultiplierImpl(factor)) {}
Multiplier::~Multiplier() { delete impl; }

// StringProcessorImpl implementation
StringProcessorImpl::StringProcessorImpl() {}
StringProcessorImpl::~StringProcessorImpl() {}
void StringProcessorImpl::set_target(const char* t) {
    target = t;
}
int StringProcessorImpl::count_char(char ch) {
    int count = 0;
    for (char c : target) {
        if (c == ch) count++;
    }
    return count;
}

// StringProcessor implementation
StringProcessor::StringProcessor() : impl(new StringProcessorImpl()) {}
StringProcessor::~StringProcessor() { delete impl; }
