# 特性示例：实例字段（FieldDecl）

## 背景

C++ 类的 `public` 非静态数据成员（实例字段）会被 cpp2rust-demo 提取为
`#[cpp(field = "ClassName::field_name")]` 访问器绑定，嵌入 `import_class!` 块中：

- **可变字段**：生成 getter（`&self`）+ setter（`&mut self`）两个访问器
- **const 字段**：仅生成 getter（`&self`）

访问器名称为 `get_<rust_field_name>` 和 `get_<rust_field_name>_mut`，
遵循与方法名相同的 snake_case 转换规则。

## 源码文件

- `instance_fields.hpp`：`Point` 结构体，含 `x`、`y`（可变）和 `id`（const）字段
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat07 --link point \
    -- clang -x c++ -fsyntax-only examples/features/07-instance-fields/entry.cpp

cpp2rust-demo merge --feature feat07
cat .cpp2rust/feat07/rust/src/lib.rs
```

## 预期生成结果

```rust
hicc::import_class! {
    #[cpp(class = "Point", ctor = "Point(int, double, double)")]
    class Point {
        // 可变字段 x：getter + setter
        #[cpp(field = "Point::x")]
        fn get_x(&self) -> &f64;
        #[cpp(field = "Point::x")]
        fn get_x_mut(&mut self) -> &mut f64;

        // 可变字段 y：getter + setter
        #[cpp(field = "Point::y")]
        fn get_y(&self) -> &f64;
        #[cpp(field = "Point::y")]
        fn get_y_mut(&mut self) -> &mut f64;

        // const 字段 id：仅 getter
        #[cpp(field = "Point::id")]
        fn get_id(&self) -> &i32;

        // 普通方法
        #[cpp(method = "double length() const")]
        fn length(&self) -> f64;
    }
}
```

## 关键结论

| C++ 字段 | 访问器 | 类型 |
|---------|-------|------|
| `double x` | `get_x(&self)` + `get_x_mut(&mut self)` | `&f64` / `&mut f64` |
| `double y` | `get_y(&self)` + `get_y_mut(&mut self)` | `&f64` / `&mut f64` |
| `const int id` | `get_id(&self)` 仅只读 | `&i32` |

> 字段访问器以引用形式返回，允许直接赋值（通过 `*point.get_x_mut() = 1.0`）。
