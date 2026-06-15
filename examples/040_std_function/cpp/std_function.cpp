#include "std_function.h"
#include <iostream>
#include <functional>
#include <vector>
#include <thread>
#include <chrono>

// CallbackWrapperImpl implementation
CallbackWrapperImpl::CallbackWrapperImpl(int (*fn)(int)) : callback(fn) {}
CallbackWrapperImpl::~CallbackWrapperImpl() {}
int CallbackWrapperImpl::invoke(int value) {
    if (callback) return callback(value);
    return value;
}
void CallbackWrapperImpl::set(int (*fn)(int)) {
    callback = fn;
}

// ProcessorImpl implementation
ProcessorImpl::ProcessorImpl() : callback(nullptr) {}
ProcessorImpl::~ProcessorImpl() {}
void ProcessorImpl::set_callback(int (*cb)(int)) {
    callback = cb;
}
int ProcessorImpl::process(int value) {
    if (callback) return callback(value);
    return value;
}

// MultiCallbackImpl implementation
MultiCallbackImpl::MultiCallbackImpl() {}
MultiCallbackImpl::~MultiCallbackImpl() {}
void MultiCallbackImpl::add(int (*cb)(int)) {
    callbacks.push_back(cb);
}
void MultiCallbackImpl::invoke_all(int value) {
    for (auto& cb : callbacks) {
        cb(value);
    }
}

// AsyncProcessorImpl implementation
AsyncProcessorImpl::AsyncProcessorImpl() : cancelled(false) {}
AsyncProcessorImpl::~AsyncProcessorImpl() {}
void AsyncProcessorImpl::set_completion_callback(void (*cb)(int, int)) {
    completion_callback = cb;
}
void AsyncProcessorImpl::set_progress_callback(void (*cb)(int)) {
    progress_callback = cb;
}
void AsyncProcessorImpl::start(int value) {
    cancelled = false;
    for (int i = 0; i <= 100; i += 20) {
        if (cancelled) break;
        if (progress_callback) progress_callback(i);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    if (completion_callback) completion_callback(value, value * 2);
}
void AsyncProcessorImpl::cancel() {
    cancelled = true;
}

// CallbackWrapper implementation
CallbackWrapper::CallbackWrapper(int (*fn)(int)) : impl(new CallbackWrapperImpl(fn)) {}
CallbackWrapper::~CallbackWrapper() { delete impl; }

// Processor implementation
Processor::Processor() : impl(new ProcessorImpl()) {}
Processor::~Processor() { delete impl; }

// MultiCallback implementation
MultiCallback::MultiCallback() : impl(new MultiCallbackImpl()) {}
MultiCallback::~MultiCallback() { delete impl; }

// AsyncProcessor implementation
AsyncProcessor::AsyncProcessor() : impl(new AsyncProcessorImpl()) {}
AsyncProcessor::~AsyncProcessor() { delete impl; }
