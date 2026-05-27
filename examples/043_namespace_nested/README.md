# 043_namespace_nested - 嵌套命名空间

## C++ 特性

本示例展示 C++ 嵌套命名空间（`foo::bar::config::ConfigManager`）如何在 FFI 中处理，以及命名空间如何影响符号名称。

## 重要说明

由于 hicc 的 `import_class!` 宏不支持嵌套命名空间类（如 `foo::bar::config::ConfigManager`），本示例使用 **raw extern "C" + void\*** 模式来实现 FFI。

## C++ 代码

### namespace_nested.h

```cpp
#pragma once

#include <cstddef>
#include <cstring>

#ifdef __cplusplus
extern "C" {
#endif

// FFI opaque pointers using void*
void* config_manager_new(void);
void config_manager_delete(void* self);
void config_manager_set_value(void* self, const char* key, int value);
int config_manager_get_value(void* self, const char* key);

int string_length(const char* str);

void* data_processor_new(void);
void data_processor_delete(void* self);
int data_processor_process(void* self, int input);

const char* get_version(void);
int get_build_number(void);

#ifdef __cplusplus
}

// Full class definitions
namespace foo {
    namespace bar {
        namespace config {
            class ConfigManager {
            public:
                static constexpr size_t MAX_ENTRIES = 10;
            private:
                int values_[MAX_ENTRIES];
                const char* keys_[MAX_ENTRIES];
                size_t count_;
            public:
                ConfigManager();
                ~ConfigManager();
                void set_value(const char* key, int value);
                int get_value(const char* key) const;
            };
        }
    }

    namespace baz {
        class DataProcessor {
        private:
            int multiplier_;
        public:
            DataProcessor();
            ~DataProcessor();
            int process(int input) const;
        };
    }
}

#endif
```

### namespace_nested.cpp

```cpp
#include "namespace_nested.h"

namespace foo {
    namespace bar {
        namespace config {
            ConfigManager::ConfigManager() : count_(0) { /* ... */ }
            // ... 方法实现
        }
    }

    namespace baz {
        DataProcessor::DataProcessor() : multiplier_(1) {}
        int DataProcessor::process(int input) const {
            return input * multiplier_;
        }
    }
}

// FFI wrapper functions - 使用 void* 作为 opaque pointer
void* config_manager_new() {
    return new foo::bar::config::ConfigManager();
}

void config_manager_delete(void* self) {
    if (self) {
        delete static_cast<foo::bar::config::ConfigManager*>(self);
    }
}
```

## Rust FFI 代码

### main.rs

```rust
// 使用 void* opaque pointer 模式
type OperationResult = *mut std::ffi::c_void;

#[link(name = "namespace_nested")]
unsafe extern "C" {
    fn config_manager_new() -> ConfigManager;
    fn config_manager_delete(p: ConfigManager);
    fn config_manager_set_value(p: ConfigManager, key: *const i8, value: i32);
    fn config_manager_get_value(p: ConfigManager, key: *const i8) -> i32;
    fn data_processor_new() -> DataProcessor;
    fn data_processor_delete(p: DataProcessor);
    fn data_processor_process(p: DataProcessor, input: i32) -> i32;
}
```

## 嵌套命名空间 FFI 模式

| 方案 | 优点 | 缺点 |
|------|------|------|
| import_class! + 嵌套类 | 简洁 | hicc 不支持嵌套命名空间类 |
| raw extern "C" + void\* | 兼容性好 | 需要手动类型转换 |

## 符号名称分析

| C++ 命名空间 | 符号名（Linux） | 说明 |
|-------------|----------------|------|
| `foo::bar::config::ConfigManager` | `_ZN3foo3bar6config13ConfigManagerC1Ev` | 构造函数（mangled） |
| `extern "C"` 函数 | `config_manager_new` | flat 符号名 |

## 构建方法

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
=== 043_namespace_nested - 嵌套命名空间 ===

--- foo::bar::config::ConfigManager ---
timeout = 30
retry = 3
port = 8080

--- string_length ---
string_length("Hello, World!") = 13

--- foo::baz::DataProcessor ---
process(42) = 42

--- Top-level Functions ---
version = 1.0.0
build_number = 42

--- 总结 ---
1. C++ 嵌套命名空间：foo::bar::config
2. 命名空间影响符号名称
3. FFI 声明使用完全限定名称
4. Rust 端使用 opaque pointer 模式
5. hicc import_class! 不支持嵌套命名空间，使用 raw extern "C"
```

## 总结

1. C++ 嵌套命名空间可以任意深度嵌套（如 `foo::bar::config`）
2. `extern "C"` 接口使用 flat 符号名，不包含命名空间信息
3. **hicc 的 `import_class!` 不支持嵌套命名空间类**
4. 使用 **void\* + static_cast** 模式实现 opaque pointer FFI
5. Rust 端使用 `*mut std::ffi::c_void` 作为 opaque pointer 类型
