# ⚙️ 半自动示例 01：`dynamic_cast` 向下转型绑定

**汇总统计类别：⚙️ 半自动**（工具生成注释骨架，用户解注释或加 flag 后可完全自动化）

---

## 背景

C++ 中常通过基类指针持有不同子类对象，在确认子类型后用 `dynamic_cast` 做向下转型：

```cpp
Animal* animal = get_animal();
if (Dog* dog = dynamic_cast<Dog*>(animal)) {
    dog->fetch("ball");
}
```

hicc 支持 `dynamic_cast` 绑定，但 cpp2rust-demo 默认以**注释形式**生成这部分绑定（避免引入非必要的 RTTI 开销），用户确认需要后解注释即可。

---

## C++ 源码（`animals.hpp` / `entry.cpp`）

```cpp
// animals.hpp
class Animal {
public:
    virtual ~Animal() {}
    virtual const char* speak() const = 0;
    virtual const char* kind()  const = 0;
};

class Dog : public Animal {
public:
    explicit Dog(const char* name);
    const char* speak() const override;  // "Woof"
    const char* kind()  const override;  // "Dog"
    const char* name()  const;
    void fetch(const char* item) const;  // Dog-specific
};

class Cat : public Animal {
public:
    explicit Cat(const char* name);
    const char* speak() const override;  // "Meow"
    const char* kind()  const override;  // "Cat"
    const char* name()  const;
    void purr() const;                   // Cat-specific
};
```

---

## 运行步骤

在仓库根目录执行：

```bash
# 第 1 步：生成分组 FFI
cpp2rust-demo init --feature sa01 --link animals \
    -- clang -x c++ -fsyntax-only examples/semi-auto/01-dynamic-cast/entry.cpp

# 第 2 步：合并
cpp2rust-demo merge --feature sa01

# 第 3 步：查看包含 dynamic_cast 注释骨架的产物
cat .cpp2rust/sa01/rust/src/free/dynamic_casts.rs
```

> **非交互环境**（CI）会自动全选中间件文件；交互终端下按 `Space` 勾选、`Enter` 确认。

---

## 预期生成产物

### `method/mtd_entry.rs`（正常方法绑定，✅ 完全自动）

```rust
// Animal — 全纯虚接口
hicc::import_class! {
    #[interface]
    class Animal {
        #[cpp(method = "const char * speak() const")]
        fn speak(&self) -> *const i8;

        #[cpp(method = "const char * kind() const")]
        fn kind(&self) -> *const i8;
    }
}

// Dog — 继承 Animal，含 Dog-specific 方法
hicc::import_class! {
    class Dog: Animal {
        ctor = "Dog(const char *)"

        #[cpp(method = "const char * speak() const")]
        fn speak(&self) -> *const i8;

        #[cpp(method = "const char * kind() const")]
        fn kind(&self) -> *const i8;

        #[cpp(method = "const char * name() const")]
        fn name(&self) -> *const i8;

        #[cpp(method = "void fetch(const char *) const")]
        fn fetch(&self, item: *const i8);
    }
}

// Cat — 类似 Dog
hicc::import_class! {
    class Cat: Animal {
        ctor = "Cat(const char *)"

        #[cpp(method = "const char * speak() const")]
        fn speak(&self) -> *const i8;

        #[cpp(method = "const char * kind() const")]
        fn kind(&self) -> *const i8;

        #[cpp(method = "const char * name() const")]
        fn name(&self) -> *const i8;

        #[cpp(method = "void purr() const")]
        fn purr(&self);
    }
}
```

### `free/dynamic_casts.rs`（⚙️ 注释骨架）

```rust
// dynamic_cast 绑定（默认注释，解注释后可用）
//
// 用法：
//   let dog: Option<hicc::ClassRef<'_, Dog>> = animal.dynamic_cast::<Dog>();

// @dynamic_cast Animal → Dog
// hicc::dynamic_cast! {
//     fn animal_to_dog(src: &Animal) -> Option<&Dog>;
// }

// @dynamic_cast Animal → Cat
// hicc::dynamic_cast! {
//     fn animal_to_cat(src: &Animal) -> Option<&Cat>;
// }
```

---

## 解锁方式

### 方式一：手动解注释

打开 `.cpp2rust/sa01/rust/src/free/dynamic_casts.rs`，去掉目标 cast 对的注释符即可。

### 方式二：`--enable-dynamic-cast` flag（推荐）

```bash
cpp2rust-demo init --feature sa01 --link animals --enable-dynamic-cast \
    -- clang -x c++ -fsyntax-only examples/semi-auto/01-dynamic-cast/entry.cpp
```

加上该 flag 后，`dynamic_casts.rs` 中的绑定直接生成为可编译的代码，无需解注释。

---

## 转换流程手册

```
C++ 类层次
    │  clang AST 解析
    ▼
CXXRecordDecl (Animal / Dog / Cat)
    │  cpp2rust-demo 提取
    ▼
ClassIR { bases: ["Animal"], ... }
    │  codegen
    ▼
method/mtd_entry.rs  ──────── 正常继承绑定 (✅ 全自动)
free/dynamic_casts.rs ─────── dynamic_cast 注释骨架 (⚙️ 半自动)
    │
    │  用户解注释 / 加 --enable-dynamic-cast flag
    ▼
可编译的 hicc dynamic_cast! 绑定
```

---

## 场景解析

| 步骤 | 工具行为 | 用户操作 |
|------|---------|---------|
| `init` | 检测 `dynamic_cast` 可能路径（基类/子类关系） | 无 |
| codegen | 在 `free/dynamic_casts.rs` 生成**注释**骨架 | 无 |
| 解锁 | — | 解注释或加 `--enable-dynamic-cast` flag |
| 使用 | Rust 侧调用 `animal.dynamic_cast::<Dog>()` | 实现业务逻辑 |

---

## 限制说明

| 限制 | 说明 |
|------|------|
| 虚继承 / 菱形继承 | `dynamic_cast` 语义不确定，工具跳过生成 |
| 多重继承 | 当前仅处理首个 public 基类，多基类的 cast 路径不完整 |
| `static_cast` | 不生成，用户需要时直接在 Rust unsafe 块中手写 |
