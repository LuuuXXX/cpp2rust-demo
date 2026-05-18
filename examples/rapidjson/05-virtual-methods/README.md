# 场景 05：非纯虚方法 → `import_class!`（vtable 透明调用）

本示例演示 cpp2rust-demo 处理**非纯虚方法**（`virtual` 但有默认实现）的流程：
这些方法与普通方法完全一样被提取，hicc 通过 vtable 透明调用，
Rust 端无需区分虚函数与非虚函数。

---

## 背景

RapidJSON 的 `CrtAllocator` 是一个 "policy" 类——没有虚函数，
但很多基于 RapidJSON 的库会将 allocator 抽象为可覆写的虚函数接口。

本示例用 `BaseAllocator` 演示这种常见模式，对比场景 04（全纯虚）与本场景（有默认实现的 virtual）的处理差异。

---

## C++ 源码（`virtual_allocator.hpp`）

```cpp
class BaseAllocator {
public:
    BaseAllocator();
    virtual ~BaseAllocator() {}           // 虚析构（跳过）

    virtual void* Malloc(size_t size);    // 非纯虚，有默认实现
    virtual void* Realloc(void* p, size_t old_size, size_t new_size);
    virtual void  Free(void* ptr);
    virtual bool  CanFree() const;        // const 虚方法

    // 非虚辅助方法
    size_t align(size_t size, size_t alignment) const;

    // 静态成员（进入 import_lib!）
    static const bool kNeedFree;
};
```

所有 public 方法均有实现（非纯虚），析构函数是虚的但无显式绑定。

---

## 运行步骤

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature rj05 --link allocator \
    -- clang -x c++ -fsyntax-only examples/rapidjson/05-virtual-methods/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature rj05

# 第 3 步：查看产物
cat .cpp2rust/rj05/rust/src/lib.rs
```

---

## 预期生成产物

### `method/mtd_entry.rs`

```rust
hicc::import_class! {
    #[cpp(class = "BaseAllocator", ctor = "BaseAllocator()")]
    class BaseAllocator {
        // 非纯虚方法 — 与普通方法完全相同提取
        #[cpp(method = "void * Malloc(size_t)")]
        fn malloc(&mut self, size: usize) -> *mut core::ffi::c_void;

        #[cpp(method = "void * Realloc(void *, size_t, size_t)")]
        fn realloc(
            &mut self,
            original_ptr: *mut core::ffi::c_void,
            original_size: usize,
            new_size: usize,
        ) -> *mut core::ffi::c_void;

        #[cpp(method = "void Free(void *)")]
        fn free(&mut self, ptr: *mut core::ffi::c_void);

        #[cpp(method = "bool CanFree() const")]
        fn can_free(&self) -> bool;

        // 非虚方法 — 同样提取
        #[cpp(method = "size_t align(size_t, size_t) const")]
        fn align(&self, size: usize, alignment: usize) -> usize;
    }
}
```

### `free/fn_entry.rs`（静态成员）

```rust
hicc::import_lib! {
    #![link_name = "allocator"]

    class BaseAllocator;

    // Static data member
    #[cpp(data = "BaseAllocator::kNeedFree")]
    fn base_allocator_k_need_free() -> &'static bool;
}
```

---

## 场景解析

### 1. 非纯虚方法的提取路径

在 `extract_class_body()` 的方法扫描循环中，对每个 `CXXMethodDecl` 节点：

```
if is_pure_virtual(child):
    → pure_virtual_nodes.push(child)  // 单独收集
else if is_public:
    → 正常提取为 FunctionIR          // 与普通方法相同路径
```

`Malloc`、`Realloc`、`Free`、`CanFree` 和 `align` 均走**正常提取路径**，
无论是否有 `virtual` 关键字，对最终生成的 Rust 代码没有影响。

### 2. hicc 通过 vtable 透明调用

从 hicc 的角度，`import_class!` 中的方法绑定不区分虚/非虚：

- hicc 生成的 C++ 侧适配代码调用 `obj->Malloc(size)`
- C++ 编译器根据对象的实际类型通过 vtable 分发
- 若对象是 `BaseAllocator` 实例，调用 `BaseAllocator::Malloc`
- 若对象是子类实例，调用子类的 `Malloc`（多态）

**Rust 端对 vtable 完全透明**，无需显式处理。

### 3. 与场景 04 的对比

| 特性 | 场景 04（全纯虚）| 场景 05（非纯虚）|
|------|----------------|----------------|
| 判定条件 | `is_abstract = true` | 普通类 |
| 生成方式 | `#[interface]` trait | 普通 `import_class!` |
| `@make_proxy` | ✅ 自动生成 | ❌ 不生成（有默认实现）|
| 构造函数 | ❌ 不生成（接口无实例）| ✅ `ctor = "BaseAllocator()"` |
| Rust 端创建实例 | 通过 `new_xxx_proxy()` | 直接 `BaseAllocator::new()`（hicc 构造函数绑定）|

### 4. 静态成员常量的提取

`static const bool kNeedFree` 是类静态数据成员，
cpp2rust-demo 将其提取为 `GlobalVarIR`（`class_name = Some("BaseAllocator")`），
并在 `import_lib!` 中生成 `#[cpp(data = "BaseAllocator::kNeedFree")]` 绑定。

`const` 成员生成 `&'static bool`（只读引用）。

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 虚析构函数 | `~BaseAllocator()` 跳过（`HiccLimitation`）|
| `size_t` 参数 | 映射为 `usize`，平台相关 |
| `void*` 参数 | 映射为 `*mut core::ffi::c_void`，需 `unsafe` |
| 非 public 虚方法 | `protected virtual` 方法不提取（自动跳过非 public 成员）|
