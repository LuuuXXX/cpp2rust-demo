# 045_union_basic - union（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **union** 的 tagged union 与内存 overlay 基本操作。采用 idiomatic 命名空间风格
（`union_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`Variant` / `IntFloatUnion` 直接持有 union，演示 set/get 与 int/float 共享存储。
析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### union_basic.h

```cpp
namespace union_basic_ns {

class Variant {
    int type_;
    union { int i_; float f_; char s_[64]; };
    std::string sbuf_;
public:
    Variant();
    void set_int(int v);
    void set_float(float v);
    void set_string(const char* v);
    int get_type() const;
    int get_int() const;
    float get_float() const;
    const char* get_string() const;
};

class IntFloatUnion {
    union { int i_; float f_; };
public:
    IntFloatUnion();
    void set_int(int v);
    void set_float(float v);
    int get_int() const;
    float get_float() const;
};

} // namespace union_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "union_basic_ns::Variant")]
    pub class Variant {
        #[cpp(method = "void set_int(int)")]
        pub fn set_int(&mut self, v: i32);
        #[cpp(method = "const char* get_string() const")]
        pub fn get_string(&self) -> *const i8;
        // set_float / set_string / get_type / get_int / get_float 略

        pub fn new() -> Self { variant_new() }
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    #[cpp(func = "std::unique_ptr<union_basic_ns::Variant> hicc::make_unique<union_basic_ns::Variant>()")]
    pub fn variant_new() -> Variant;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| tagged union | `type_` + `union` 成员 | hicc 绑定内部持有，对外透明 |
| 字符串 | `char[64]` + `std::string` backing | `*const i8` + `CStr` |
| overlay | `union { int; float; }` | 同名方法读取共享 bits |
| 析构 | `~Variant` / `~IntFloatUnion` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 045_union_basic - union（hicc 直出）===

variant int type=0 value=42
variant float type=1 value=2.5
variant string type=2 value=hi
union int=7
union float=1.5

Rust FFI: hicc 直接绑定持有 union 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_variant_int` | set_int / get_type / get_int |
| `smoke_variant_float` | set_float / get_type / get_float |
| `smoke_variant_string` | set_string / get_type / get_string |
| `smoke_int_float_union_int` | set_int / get_int |
| `smoke_int_float_union_float` | set_float / get_float |

### 运行方式

```bash
cd examples/045_union_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ union 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 跨 FFI 只交换标量与 `const char*`，字符串返回通过内部 backing 保持 `c_str()` 稳定
