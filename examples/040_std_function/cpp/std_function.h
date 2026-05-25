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
void callback_wrapper_delete(struct CallbackWrapper* self);

int callback_wrapper_invoke(const struct CallbackWrapper* self, int value);
void callback_wrapper_set(struct CallbackWrapper* self, int (*fn)(int));

// Processor structure
struct Processor;

struct Processor* processor_new(void);
void processor_delete(struct Processor* self);

void processor_set_callback(struct Processor* self, int (*cb)(int));
int processor_process(const struct Processor* self, int value);

// MultiCallback structure
struct MultiCallback;

struct MultiCallback* multi_callback_new(void);
void multi_callback_delete(struct MultiCallback* self);

void multi_callback_add(struct MultiCallback* self, int (*cb)(int));
void multi_callback_invoke_all(struct MultiCallback* self, int value);

// AsyncProcessor
struct AsyncProcessor;

struct AsyncProcessor* async_processor_new(void);
void async_processor_delete(struct AsyncProcessor* self);

void async_processor_set_callback(struct AsyncProcessor* self, void (*cb)(int, int));
void async_processor_set_progress_callback(struct AsyncProcessor* self, void (*cb)(int));

void async_processor_start(struct AsyncProcessor* self, int value);
void async_processor_cancel(struct AsyncProcessor* self);

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
};

struct Processor {
    ProcessorImpl* impl;
    Processor();
    ~Processor();
};

struct MultiCallback {
    MultiCallbackImpl* impl;
    MultiCallback();
    ~MultiCallback();
};

struct AsyncProcessor {
    AsyncProcessorImpl* impl;
    AsyncProcessor();
    ~AsyncProcessor();
};

#endif
