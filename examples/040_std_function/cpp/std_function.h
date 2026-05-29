#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// std::function 回调示例
// 展示如何通过 FFI 传递 Rust 闭包到 C++

#include <stddef.h>

// Callback wrapper
struct CallbackWrapper;

struct CallbackWrapper* callback_wrapper_new(int (*fn)(int));
struct CallbackWrapper* callback_wrapper_new_double(void);
void callback_wrapper_delete(struct CallbackWrapper* self);

// Processor structure
struct Processor;

struct Processor* processor_new(void);
void processor_set_double(struct Processor* p);
void processor_delete(struct Processor* self);

// MultiCallback structure
struct MultiCallback;

struct MultiCallback* multi_callback_new(void);
void multi_callback_add_double(struct MultiCallback* mc);
void multi_callback_add_triple(struct MultiCallback* mc);
void multi_callback_delete(struct MultiCallback* self);

// AsyncProcessor
struct AsyncProcessor;

struct AsyncProcessor* async_processor_new(void);
void async_processor_delete(struct AsyncProcessor* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <functional>
#include <vector>

class CallbackWrapperImpl {
public:
    std::function<int(int)> callback;
    explicit CallbackWrapperImpl(int (*fn)(int));
    ~CallbackWrapperImpl();
    int invoke(int value);
    void set(int (*fn)(int));
};

class ProcessorImpl {
public:
    std::function<int(int)> callback;
    ProcessorImpl();
    ~ProcessorImpl();
    void set_callback(int (*cb)(int));
    int process(int value);
};

class MultiCallbackImpl {
public:
    std::vector<std::function<int(int)>> callbacks;
    MultiCallbackImpl();
    ~MultiCallbackImpl();
    void add(int (*cb)(int));
    void invoke_all(int value);
};

class AsyncProcessorImpl {
public:
    std::function<void(int, int)> completion_callback;
    std::function<void(int)> progress_callback;
    bool cancelled;
    AsyncProcessorImpl();
    ~AsyncProcessorImpl();
    void set_completion_callback(void (*cb)(int, int));
    void set_progress_callback(void (*cb)(int));
    void start(int value);
    void cancel();
};

struct CallbackWrapper {
    CallbackWrapperImpl* impl;
    explicit CallbackWrapper(int (*fn)(int));
    ~CallbackWrapper();
    int invoke(int value) { return impl->invoke(value); }
};

struct Processor {
    ProcessorImpl* impl;
    Processor();
    ~Processor();
    int process(int value) { return impl->process(value); }
};

struct MultiCallback {
    MultiCallbackImpl* impl;
    MultiCallback();
    ~MultiCallback();
    void invoke_all(int value) { impl->invoke_all(value); }
};

struct AsyncProcessor {
    AsyncProcessorImpl* impl;
    AsyncProcessor();
    ~AsyncProcessor();
    bool is_cancelled() const { return impl->cancelled; }
    void cancel() { impl->cancel(); }
};

#endif
