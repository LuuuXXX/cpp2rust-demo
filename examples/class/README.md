# 类示例：包含成员方法的 C++ 类

该示例演示如何从 C++ 类声明/实现生成 Rust FFI，并观察实例方法与静态方法的分层输出。

## 源码文件

- `widget.hpp`：类声明
- `widget.cpp`：类实现

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI（用独立 feature，避免和其他示例混合）
cpp2rust-demo init --feature widget --link widget -- clang -x c++ -fsyntax-only examples/class/widget.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature widget

# 第 3 步：查看结果
cat .cpp2rust/widget/rust/src/merged_ffi.rs
```

> 说明：交互终端下，`init` 会提示选择要纳入转换的中间件文件（`Space` 勾选，`Enter` 确认）；非交互环境会自动全选。

## 预期生成结果

```rust
// 实例方法进入 import_class!
hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);

        #[cpp(method = "int getId() const")]
        fn get_id(&self) -> i32;

        #[cpp(method = "bool isVisible() const")]
        fn is_visible(&self) -> bool;
    }
}

// 静态方法与前置声明进入 import_lib!
hicc::import_lib! {
    #![link_name = "widget"]

    class Widget;

    #[cpp(func = "int Widget::instanceCount()")]
    fn widget_instance_count() -> i32;
}
```

## hicc 中类相关绑定的映射方式

- 实例方法：`import_class!` + `#[cpp(method = "...")]`
- 静态方法：`import_lib!` + `#[cpp(func = "...")]`
- 类前置声明：`import_lib!` 中的 `class Widget;`

## 当前限制

| 能力项 | 状态 |
|---------|--------|
| public 实例方法 | ✅ 已支持 |
| `const` 方法 | ✅ 已支持（映射为 `&self`） |
| `static` 方法 | ✅ 已支持（映射到 `import_lib!`） |
| 构造/析构函数 | ⚠️ 当前跳过，建议用工厂函数 |
| private/protected 成员 | ✅ 自动跳过 |
| virtual / pure-virtual 方法 | ⚠️ 当前跳过，可手工补充 |
| 继承 | ❌ 暂不支持 |
| 模板 | ❌ 暂不支持 |
| 运算符重载 | ❌ 暂不支持（受 hicc 能力限制） |
