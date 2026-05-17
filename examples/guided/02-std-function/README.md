# 🔧 引导支持示例 02：`std::function` / lambda 回调参数

**汇总统计类别：🔧 引导支持**（工具生成接口骨架，用户必须手写 C++ 接口类 + Rust `impl`）

---

## 背景

Rust 没有与 C++ `std::function<R(Args...)>` 兼容的 ABI。
当 C++ 方法接受 `std::function` 参数时，hicc 无法直接生成绑定。

cpp2rust-demo 的解决方案：

1. **跳过** 含 `std::function` 参数的方法，在接口报告中生成**纯虚接口类骨架**。
2. 用户手写对应的 C++ 纯虚接口类（`EventHandler`），将 `std::function` 替换为虚函数调用。
3. 用 `@make_proxy` 反向绑定，让 Rust 实现该接口并传回 C++。

这与全纯虚抽象类的 `@make_proxy` 机制完全相同（见 `examples/rapidjson/04-abstract-interface/`）。

---

## C++ 源码（`event_emitter.hpp` / `entry.cpp`）

```cpp
// event_emitter.hpp
#include <functional>

class EventEmitter {
public:
    EventEmitter();

    // ── std::function 参数 → SKIPPED ─────────────────────────────────
    void on_event(std::function<void(int)> handler);
    void on_message(std::function<void(int, const char*)> handler);

    // ── POD 方法 → 自动提取 ✅ ────────────────────────────────────────
    void emit(int event_id) const;
    int  handler_count() const;
};
```

---

## 运行步骤

```bash
# 第 1 步：生成 FFI（on_event/on_message 被跳过，emit/handler_count 自动提取）
cpp2rust-demo init --feature g02 --link event_emitter \
    -- clang -x c++ -fsyntax-only examples/guided/02-std-function/entry.cpp

# 查看接口报告（含接口骨架建议）
cat .cpp2rust/g02/meta/init-interface-report.md
```

---

## 接口报告骨架（工具自动生成）

```
## Guided Support: std::function Parameters

以下方法因含 std::function 参数而跳过。
建议按如下步骤处理：

1. 为每个 std::function<R(Args...)> 参数定义一个纯虚接口类：

   // EventHandler — 替代 std::function<void(int)>
   class EventHandler {
   public:
       virtual ~EventHandler() {}
       virtual void on_event(int event_id) = 0;
   };

   // MessageHandler — 替代 std::function<void(int, const char*)>
   class MessageHandler {
   public:
       virtual ~MessageHandler() {}
       virtual void on_message(int event_id, const char* payload) = 0;
   };

2. 修改 EventEmitter，用接口类替换 std::function 参数：

   void on_event(EventHandler* handler);
   void on_message(MessageHandler* handler);

3. 在新的 entry2.cpp 中引入上述接口，重跑 cpp2rust-demo init。
   工具将自动生成 #[interface] + @make_proxy 绑定。

4. Rust 侧实现接口 trait：

   struct MyHandler;
   impl EventHandlerInterface for MyHandler {
       fn on_event(&mut self, event_id: i32) {
           println!("event: {}", event_id);
       }
   }
```

---

## 预期生成产物（重跑后）

### `entry2.rs` (methods section)（重跑后，含接口类）

```rust
// EventHandler 全纯虚接口
hicc::import_class! {
    #[interface]
    class EventHandler {
        #[cpp(method = "void on_event(int)")]
        fn on_event(&mut self, event_id: i32);
    }
}

// MessageHandler 全纯虚接口
hicc::import_class! {
    #[interface]
    class MessageHandler {
        #[cpp(method = "void on_message(int, const char *)")]
        fn on_message(&mut self, event_id: i32, payload: *const i8);
    }
}

// EventEmitter（方法替换后）
hicc::import_class! {
    class EventEmitter {
        ctor = "EventEmitter()"

        #[cpp(method = "void on_event(EventHandler *)")]
        fn on_event(&mut self, handler: *mut EventHandler);

        #[cpp(method = "void on_message(MessageHandler *)")]
        fn on_message(&mut self, handler: *mut MessageHandler);

        #[cpp(method = "void emit(int) const")]
        fn emit(&self, event_id: i32);

        #[cpp(method = "int handler_count() const")]
        fn handler_count(&self) -> i32;
    }
}
```

### `entry2.rs` (free functions section)（`@make_proxy` 反向绑定）

```rust
hicc::import_lib! {
    #![link_name = "event_emitter"]

    // @make_proxy 让 Rust struct 实现 EventHandler 接口
    #[cpp(make_proxy = "EventHandler")]
    fn make_event_handler_proxy() -> *mut EventHandler;

    #[cpp(make_proxy = "MessageHandler")]
    fn make_message_handler_proxy() -> *mut MessageHandler;
}
```

---

## Rust 侧使用示例

```rust
// 实现接口 trait
struct MyHandler;
impl EventHandlerInterface for MyHandler {
    fn on_event(&mut self, event_id: i32) {
        println!("Received event: {}", event_id);
    }
}

// 构造代理并注册
let proxy = make_event_handler_proxy(Box::new(MyHandler));
emitter.on_event(proxy);
emitter.emit(42);
```

---

## 转换流程手册

```
C++ 方法（含 std::function）
    │  clang AST 解析
    ▼
CXXMethodDecl（参数含 std::function<void(int)>）
    │  is_supported_cpp_type 检查
    ▼
std::function 不被支持 → HiccLimitation
    │  接口报告生成纯虚接口骨架建议
    ▼
meta/init-interface-report.md（含接口类代码建议）

────── 用户手写接口类，修改 C++ 源码，重跑 ──────

    │  clang AST 解析（含接口类）
    ▼
CXXRecordDecl（EventHandler，全纯虚）
    │  cpp2rust-demo 提取
    ▼
ClassIR { has_pure_virtual: true, ... }
    │  codegen
    ▼
#[interface] trait + @make_proxy 绑定（✅ 全自动）
```

---

## 场景解析

| 步骤 | 工具行为 | 用户操作 |
|------|---------|---------|
| `init` | 检测 `std::function`，标记 HiccLimitation | 无 |
| 接口报告 | 生成纯虚接口类代码骨架 | 无 |
| 修改 C++ | — | 手写接口类，替换 `std::function` 参数 |
| 重跑 `init` | 自动提取接口类 + `@make_proxy` | 无 |
| Rust 侧 | 骨架已就绪（`impl EventHandlerInterface for ...`） | 填写回调业务逻辑 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 接口粒度 | 每个不同的 `std::function<R(Args...)>` 签名需一个独立接口类 |
| 闭包捕获 | C++ lambda 的捕获列表无法通过接口类传递；需通过构造函数参数或成员变量传入状态 |
| 返回值 | 若 `std::function` 返回 `std::string` 等不支持类型，仍需额外 shim |
