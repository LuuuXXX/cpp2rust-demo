# ⚙️ 半自动示例 02：Placement New（Rust 内存中构造 C++ 对象）

**汇总统计类别：⚙️ 半自动**（工具生成注释骨架，用户解注释或加 flag 后可完全自动化）

---

## 背景

某些性能敏感场景需要在 Rust 分配的对齐内存中直接构造 C++ 对象（placement new），
避免额外的堆分配。hicc 通过 `@placement_new` 指令支持这一模式；
cpp2rust-demo 默认以**注释形式**生成该绑定，用户确认对齐需求后解注释即可。

---

## C++ 源码（`fixed_buffer.hpp` / `entry.cpp`）

```cpp
// fixed_buffer.hpp
class FixedBuffer {
public:
    explicit FixedBuffer(int capacity);
    ~FixedBuffer();

    int         write(const char* data, int size);
    const char* data()     const;
    int         size()     const;
    int         capacity() const;
    void        reset();
};
```

---

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature sa02 --link fixed_buffer \
    -- clang -x c++ -fsyntax-only examples/semi-auto/02-placement-new/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature sa02

# 第 3 步：查看 placement_new 注释骨架
cat .cpp2rust/sa02/rust/src/entry.rs
```

---

## 预期生成产物

### `entry.rs`（正常类绑定，✅ 完全自动）

```rust
hicc::import_class! {
    class FixedBuffer {
        ctor = "FixedBuffer(int)"

        #[cpp(method = "int write(const char *, int)")]
        fn write(&mut self, data: *const i8, size: i32) -> i32;

        #[cpp(method = "const char * data() const")]
        fn data(&self) -> *const i8;

        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "int capacity() const")]
        fn capacity(&self) -> i32;

        #[cpp(method = "void reset()")]
        fn reset(&mut self);
    }
}
```

### `entry.rs`（placement_new 注释骨架，⚙️ 末尾追加）

```rust
// placement new 绑定（默认注释，解注释后可用）
//
// 前提：调用方必须提供满足 FixedBuffer 对齐要求的 AlignedStorage。
// 使用方式：
//   let mut storage: hicc::AlignedStorage<FixedBuffer> = hicc::AlignedStorage::uninit();
//   let obj: &mut FixedBuffer = unsafe { FixedBuffer::placement_new(&mut storage, 1024) };

// @placement_new FixedBuffer
// hicc::import_lib! {
//     #![link_name = "fixed_buffer"]
//
//     #[cpp(placement_new = "FixedBuffer(int)")]
//     fn fixed_buffer_placement_new(
//         storage: *mut hicc::AlignedStorage<FixedBuffer>,
//         capacity: i32,
//     ) -> *mut FixedBuffer;
// }
```

---

## 解锁方式

### 手动解注释

打开 `.cpp2rust/sa02/rust/src/entry.rs`，去掉末尾 `placement_new` 注释骨架中目标类的注释符即可。

---

## 转换流程手册

```
C++ 类（含构造函数）
    │  clang AST 解析
    ▼
CXXRecordDecl + CXXConstructorDecl
    │  cpp2rust-demo 提取 CtorIR
    ▼
ClassIR { ctors: [CtorIR { ... }], ... }
    │  codegen
    ▼
entry.rs             ─── 正常方法绑定 (✅ 全自动)
entry.rs             ─── placement_new 注释骨架 (⚙️ 半自动，末尾追加)
    │
    │  用户解注释 / 加 --enable-placement-new flag
    ▼
可编译的 @placement_new 绑定
    │
    │  Rust 侧使用
    ▼
unsafe { FixedBuffer::placement_new(&mut storage, 1024) }
```

---

## 场景解析

| 步骤 | 工具行为 | 用户操作 |
|------|---------|---------|
| `init` | 检测含构造函数的 ClassIR | 无 |
| codegen | 在 `entry.rs` 末尾追加**注释**的 `@placement_new` 骨架 | 无 |
| 解锁 | — | 解注释 `entry.rs` 末尾 `placement_new` 骨架 |
| 使用 | Rust 侧调用 `placement_new()`，传入对齐内存 | 提供 `AlignedStorage<T>` 内存 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 对齐要求 | Rust 侧必须提供满足 `alignof(T)` 的对齐内存，否则行为未定义 |
| 析构 | placement new 创建的对象需要手动调用析构函数；hicc 代理对象不自动析构 |
| 移动语义 | placement new 不涉及移动构造；如需移动语义需额外 shim |
