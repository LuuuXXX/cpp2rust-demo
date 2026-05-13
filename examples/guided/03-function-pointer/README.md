# 🔧 引导支持示例 03：函数指针参数

**汇总统计类别：🔧 引导支持**（工具生成接口骨架，用户必须手写 C++ 接口类 + Rust `impl`）

---

## 背景

Rust 函数指针的 ABI 与 C++ 函数指针 ABI 不兼容（调用约定、异常安全等均有差异）。
hicc 不支持直接绑定带函数指针参数的 C++ 方法。

cpp2rust-demo 的解决方案与 `std::function` 相同：

1. **跳过**含函数指针参数的方法，在接口报告中生成**纯虚接口类骨架**。
2. 用户将 C 风格函数指针替换为纯虚接口类（`ICallback`、`IFilter`）。
3. 用 `@make_proxy` 让 Rust struct 实现接口并传回 C++。

---

## C++ 源码（`dispatcher.hpp` / `entry.cpp`）

```cpp
// dispatcher.hpp
typedef void (*Callback)(int event_id, void* user_data);
typedef int  (*Filter)(int event_id);

class Dispatcher {
public:
    Dispatcher();

    // ── 函数指针参数 → SKIPPED ─────────────────────────────────────────
    void register_callback(Callback cb, void* user_data);
    void set_filter(Filter filter);

    // ── POD 方法 → 自动提取 ✅ ────────────────────────────────────────
    void dispatch(int event_id) const;
    void reset();
    int  callback_count() const;
};
```

---

## 运行步骤

```bash
# 第 1 步：生成 FFI（register_callback/set_filter 被跳过）
cpp2rust-demo init --feature g03 --link dispatcher \
    -- clang -x c++ -fsyntax-only examples/guided/03-function-pointer/entry.cpp

# 查看接口报告（含接口类骨架建议）
cat .cpp2rust/g03/meta/init-interface-report.md

# 查看自动提取的 POD 方法
cat .cpp2rust/g03/rust/src/mod_entry/method/mtd_entry.rs
```

---

## 接口报告骨架（工具自动生成）

```
## Guided Support: Function Pointer Parameters

以下方法因含函数指针参数而跳过。
建议按如下步骤处理：

1. 为每个函数指针签名定义一个纯虚接口类：

   // ICallback — 替代 void(*)(int, void*)
   class ICallback {
   public:
       virtual ~ICallback() {}
       virtual void invoke(int event_id) = 0;
   };

   // IFilter — 替代 int(*)(int)
   class IFilter {
   public:
       virtual ~IFilter() {}
       virtual int accept(int event_id) = 0;
   };

2. 修改 Dispatcher，用接口类指针替换函数指针：

   void register_callback(ICallback* cb);
   void set_filter(IFilter* filter);

3. 重跑 cpp2rust-demo init（工具自动生成 #[interface] + @make_proxy 绑定）。
```

---

## 预期生成产物（重跑后）

### `method/mtd_entry2.rs`（含接口类）

```rust
// ICallback 全纯虚接口
hicc::import_class! {
    #[interface]
    class ICallback {
        #[cpp(method = "void invoke(int)")]
        fn invoke(&mut self, event_id: i32);
    }
}

// IFilter 全纯虚接口
hicc::import_class! {
    #[interface]
    class IFilter {
        #[cpp(method = "int accept(int)")]
        fn accept(&mut self, event_id: i32) -> i32;
    }
}

// Dispatcher（方法替换后）
hicc::import_class! {
    class Dispatcher {
        ctor = "Dispatcher()"

        #[cpp(method = "void register_callback(ICallback *)")]
        fn register_callback(&mut self, cb: *mut ICallback);

        #[cpp(method = "void set_filter(IFilter *)")]
        fn set_filter(&mut self, filter: *mut IFilter);

        #[cpp(method = "void dispatch(int) const")]
        fn dispatch(&self, event_id: i32);

        #[cpp(method = "void reset()")]
        fn reset(&mut self);

        #[cpp(method = "int callback_count() const")]
        fn callback_count(&self) -> i32;
    }
}
```

### `free/fn_entry2.rs`（`@make_proxy` 反向绑定）

```rust
hicc::import_lib! {
    #![link_name = "dispatcher"]

    #[cpp(make_proxy = "ICallback")]
    fn make_callback_proxy() -> *mut ICallback;

    #[cpp(make_proxy = "IFilter")]
    fn make_filter_proxy() -> *mut IFilter;
}
```

---

## Rust 侧使用示例

```rust
// 实现回调接口
struct LogCallback;
impl ICallbackInterface for LogCallback {
    fn invoke(&mut self, event_id: i32) {
        println!("Event dispatched: {}", event_id);
    }
}

// 实现过滤器接口
struct EvenFilter;
impl IFilterInterface for EvenFilter {
    fn accept(&mut self, event_id: i32) -> i32 {
        if event_id % 2 == 0 { 1 } else { 0 }
    }
}

// 注册并使用
let cb = make_callback_proxy(Box::new(LogCallback));
let filter = make_filter_proxy(Box::new(EvenFilter));
dispatcher.set_filter(filter);
dispatcher.register_callback(cb);
dispatcher.dispatch(42);  // 打印 "Event dispatched: 42"
dispatcher.dispatch(43);  // 被过滤，无输出
```

---

## 转换流程手册

```
C++ 方法（含函数指针）
    │  clang AST 解析
    ▼
CXXMethodDecl（参数含 FunctionProtoType）
    │  is_supported_cpp_type 检查
    ▼
函数指针不被支持 → HiccLimitation
    │  接口报告生成纯虚接口类建议
    ▼
meta/init-interface-report.md（含接口类代码建议）

────── 用户手写接口类，修改 C++ 源码，重跑 ──────

    │  clang AST 解析（含接口类）
    ▼
CXXRecordDecl（ICallback，全纯虚）
    │  cpp2rust-demo 提取
    ▼
ClassIR { has_pure_virtual: true, ... }
    │  codegen
    ▼
#[interface] trait + @make_proxy 绑定（✅ 全自动）
```

---

## 与 `std::function` 的对比

| 方面 | 函数指针 | `std::function` |
|------|---------|----------------|
| 跳过原因 | ABI 不兼容 | 无法映射到 Rust 闭包 |
| 接口类粒度 | 一个指针签名 → 一个接口类 | 同上 |
| 状态携带 | 需通过 `void* user_data` 传递 | 由 lambda 捕获列表携带 |
| 引导步骤 | 完全相同 | 完全相同 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| `void* user_data` | 原始 C 回调的 `user_data` 无法透传；接口类方式通过成员变量携带状态替代 |
| 多重函数指针 | 若同一方法含多个不同签名的函数指针参数，需多个接口类 |
| 异步回调 | 若 C++ 侧在另一线程调用接口，需确保 Rust `impl` 线程安全（`Send + Sync`）|
