# Direct Binding vs Shim Binding

cpp2rust-demo 根据项目 AST 自动选择绑定模式，无需手动配置。

---

## Direct Binding（直接绑定）

**适用对象**：纯 C++ 类项目——项目中有 C++ 类定义，但**没有** `extern "C"` 函数返回/接受类指针。

**工作机制**：工具检测到纯 C++ 类后，自动生成：

- `hicc::make_unique<T>` 工厂函数——创建类实例（`destroy_fn = None`，hicc 默认 `delete`）
- `#[cpp(method = "...")]`——直接绑定 C++ 类方法，无需 C shim 包装层

**生成代码示例**（`006_class_basic`）：

```rust
hicc::import_class! {
    #[cpp(class = "Counter")]
    pub struct Counter;

    #[cpp(method = "increment")]
    pub fn increment(&mut self);

    #[cpp(method = "decrement")]
    pub fn decrement(&mut self);

    #[cpp(method = "get")]
    pub fn get(&self) -> i32;
}

hicc::import_lib! {
    #[cpp(func = "std::unique_ptr<Counter> hicc::make_unique<Counter>()")]
    pub fn counter_new() -> Counter;
}
```

**优势**：零前置工作——不需要编写任何 `extern "C"` 包装层，直接绑定 C++ 类方法。

**适用范围**：006–048 大部分示例均使用 Direct 模式。

---

## Shim Binding（Shim 包装绑定）

**适用对象**：项目中有 `extern "C"` 函数返回/接受类指针（如 `Counter* counter_new()`），表明项目已存在 C-API 包装层。

**工作机制**：工具检测到 extern-C 函数引用类指针后，沿用传统 shim 流程：

- `hicc::cpp!` 块——生成 ctor/dtor/operator/static 等 C 适配层
- `*_new()` / `*_delete()` / `*_get()` ——命名 shim 函数
- `import_class!` + `import_lib!` 绑定 shim 函数

**生成代码示例**（传统风格）：

```rust
hicc::cpp! {
    extern "C" {
        fn counter_new() -> *mut Counter;
        fn counter_delete(c: *mut Counter);
        fn counter_increment(c: *mut Counter);
        fn counter_get(c: *const Counter) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    pub struct Counter;

    #[cpp(method = "counter_increment")]
    pub fn increment(&mut self);
}
```

**判定规则**：只要项目中**任何** `extern "C"` 函数的返回值或首参为类指针/引用，即触发 Shim 模式（保守策略，避免误识别不规范 C 包装为 Direct）。

**无类项目**（仅有自由函数，如 001–005）也默认使用 Shim 模式。

---

## 模式选择指南

| 项目特征 | 自动判定模式 | 说明 |
|---------|------------|------|
| 纯 C++ 类，无 extern-C | **Direct** | 推荐模式，零前置工作 |
| 有 extern-C 函数 + 类指针参数/返回值 | **Shim** | 项目已有 C 包装层，沿用 shim |
| 无类项目（仅有自由函数） | **Shim** | 保守默认，向后兼容 |
| C++ + extern-C 混合 | **Shim** | 混合场景保守走 shim |

> 模式由 `src/extractor/direct_binding.rs::classify()` 自动判定，无需手动配置。若需强制切换，可在 C++ 源文件中添加/移除 `extern "C"` 类指针函数来影响判定结果。

---

## 降级标记

两种模式均可能产生降级标记（`cpp2rust-todo[TAG]`），需手动补全：

| TAG | 原因 | 手动操作 |
|-----|------|---------|
| `[OP]` | 运算符重载 | 为命名 shim 实现 Rust `std::ops::*` trait |
| `[VA]` | 可变参数模板 | 添加新参数组合 wrapper |
| `[LM]` | Lambda/std::function | 编写 Rust→C++ trampoline |
| `[CV]` | C 可变参数函数 | 补充固定参数 wrapper |
| `[FP]` | 函数指针参数 | 确认 `extern "C"` 调用约定 |
| `[VM]` | volatile 成员函数 | 检查 volatile shim |

```bash
grep -rn "cpp2rust-todo" .cpp2rust/*/rust/src/
```
