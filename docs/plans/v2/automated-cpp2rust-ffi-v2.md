# C++ 到 Rust Safe FFI 自动化工具 - v2 计划

## 1. 背景与改进目标

### 1.1 v1 的局限

v1 版本采用**头文件解析**方式，仅能获取类型声明信息，无法处理以下场景：

| 场景 | v1 支持 | 说明 |
|------|---------|------|
| 普通类 | ✅ | 直接解析 CXXRecordDecl |
| 嵌套命名空间类 | ✅ | 需用 void* 模式 |
| 模板类声明 | ✅ | 解析 `template<typename T> class Foo` |
| **模板实例化** | ❌ | `std::vector<int>` 需要编译才能确定 |
| STL 容器 | ❌ | 无法捕获 `std::vector<int>` |
| **虚函数表** | ⚠️ | 可解析但 vtable 映射需 v2 |

**根本原因**：头文件解析只是语法分析，模板实例化发生在**编译器的语义分析阶段**。

### 1.2 v2 改进目标

1. **支持模板实例化捕获**：通过 AST 编译捕获，获取 `std::vector<int>` 等实例化类型
2. **支持 STL 容器**：自动识别并生成 hicc-std 包装
3. **支持虚函数表映射**：支持抽象类到 Rust trait
4. **保持 v1 特性**：所有 v1 功能继续支持

## 2. 技术方案

### 2.1 核心问题：为何需要 AST 编译捕获？

**C++ 编译流程**：

```
源文件 (.cpp)
    ↓
预处理阶段 (-E): 宏展开、#include 展开
    ↓  输出: .i 文件（宏已展开，但模板还是声明）
编译器前端: 语法分析、语义分析
    ↓
模板实例化: 根据实际使用生成 `std::vector<int>`
    ↓
生成目标文件 (.o)
```

**关键结论**：
- 预处理阶段：**无法**实例化模板
- 编译器语义分析阶段：**可以**实例化模板
- 需要真正的编译过程才能捕获模板实例化

### 2.2 方案对比

| 方案 | 原理 | 模板实例化支持 | 复杂度 |
|------|------|----------------|--------|
| **预处理捕获** (c2rust-demo) | LD_PRELOAD hook 拦截编译器，执行 `-E` 预处理 | ❌ 仅宏展开 | 高 |
| **头文件解析** (v1) | libclang 解析 .h 文件 | ❌ 无实例化 | 低 |
| **AST 编译捕获** (v2) | libclang 编译源文件，遍历 AST | ✅ 完整支持 | 中 |

### 2.3 v2 架构设计

```
cpp2rust-ffi tool (v2)
├── src/
│   ├── main.rs                    # CLI 入口
│   ├── parser/                   # C++ 解析
│   │   ├── mod.rs
│   │   ├── header_parser.rs      # 头文件解析 (v1 复用)
│   │   ├── ast_compiler.rs       # AST 编译捕获 (新增)
│   │   └── type_resolver.rs      # 类型解析 (新增)
│   ├── generator/                # Rust 代码生成
│   │   ├── mod.rs
│   │   ├── class_generator.rs    # 类生成 (v1 复用)
│   │   ├── template_generator.rs  # 模板实例化生成 (新增)
│   │   ├── virtual_fn_gen.rs     # 虚函数表映射 (新增)
│   │   └── project_generator.rs
│   └── template/                # 项目模板
│       └── ...
├── Cargo.toml
└── README.md
```

## 3. Examples C++ 特性覆盖详情

### 3.1 特性分类表

根据 `./examples/` 中 48 个示例的 C++ 特性分类：

#### 基础类型与函数 (1-5)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 001_hello_world | extern "C" 函数 | `FunctionDecl` | ✅ | ✅ |
| 002_function_overload | 函数重载 | `FunctionDecl` (多个同名) | ✅ | ✅ |
| 003_default_args | 默认参数 | `ParmVarDecl` (带默认值) | ✅ | ✅ |
| 004_inline_functions | 内联函数 | `FunctionDecl` + `inline` | ✅ | ✅ |
| 005_variadic_functions | 可变参数函数 | `FunctionDecl` (可变参数) | ✅ | ✅ |

#### 类与对象 (6-12)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 006_class_basic | 基础类 | `CXXRecordDecl` | ✅ | ✅ |
| 007_class_constructor | 构造/析构函数 | `CXXConstructorDecl`, `CXXDestructorDecl` | ✅ | ✅ |
| 008_class_copy | 拷贝构造函数 | `CXXConstructorDecl` (copy) | ✅ | ✅ |
| 009_class_move | 移动构造函数 | `CXXConstructorDecl` (move) | ✅ | ✅ |
| 010_class_static | 静态成员 | `VarDecl` (static) | ✅ | ✅ |
| 011_class_const | const 成员函数 | `CXXMethodDecl` (const) | ✅ | ✅ |
| 012_class_volatile | volatile 成员函数 | `CXXMethodDecl` (volatile) | ✅ | ✅ |

#### 面向对象特性 (13-18)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 013_inheritance_single | 单继承 | `CXXBaseSpecifier` | ✅ | ✅ |
| 014_inheritance_multiple | 多继承 | `CXXBaseSpecifier` (多个) | ✅ | ✅ |
| 015_virtual_basic | 虚函数基础 | `CXXMethodDecl` (virtual) | ✅ | ✅ |
| 016_virtual_pure | 纯虚函数/抽象类 | `CXXMethodDecl` (= 0) | ⚠️ | ✅ |
| 017_virtual_override | override 说明符 | `CXXMethodDecl` (override) | ✅ | ✅ |
| 018_virtual_diamond | 菱形继承 | `CXXBaseSpecifier` (virtual) | ⚠️ | ✅ |

#### 运算符与类型 (19-23)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 019_operator_overload | 运算符重载 | `CXXMethodDecl` (operator) | ❌ | ❌ |
| 020_friend_function | 友元函数 | `FriendDecl` | ❌ | ❌ |
| 021_explicit_ctor | explicit 构造函数 | `CXXConstructorDecl` (explicit) | ✅ | ✅ |
| 022_mutable_member | mutable 成员 | `FieldDecl` (mutable) | ✅ | ✅ |
| 023_typeid_rtti | typeid 与 RTTI | `CXXTypeidExpr` | ❌ | ❌ |

#### 模板 (24-28)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 024_template_function | 函数模板 | `FunctionTemplateDecl` | ✅ | ✅ |
| 025_template_class | 类模板 | `ClassTemplateDecl` | ⚠️ | ✅ |
| 026_template_specialization | 模板偏特化 | `ClassTemplatePartialSpecialization` | ❌ | ⚠️ |
| 027_template_instantiation | 模板显式实例化 | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 028_variadic_template | 可变参数模板 | `VariadicTemplate` | ❌ | ⚠️ |

#### 智能指针与内存 (29-33)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 029_unique_ptr | std::unique_ptr | `CXXNewExpr`, `TypeRef` | ✅ | ✅ |
| 030_shared_ptr | std::shared_ptr | `CXXNewExpr`, `TypeRef` | ✅ | ✅ |
| 031_custom_deleter | 自定义删除器 | `FunctionDecl` | ✅ | ✅ |
| 032_placement_new | Placement new | `CXXNewExpr` | ✅ | ✅ |
| 033_raii_pattern | RAII 模式 | 构造/析构函数 | ✅ | ✅ |

#### STL 容器 (34-38)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 034_vector_basic | std::vector | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 035_map_basic | std::map | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 036_string_basic | std::string | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 037_array_basic | std::array | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 038_tuple_basic | std::tuple | `ClassTemplateSpecialization` | ⚠️ | ✅ |

#### 函数对象 (39-42)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 039_lambda_basic | Lambda 表达式 | `LambdaExpr` | ❌ | ⚠️ |
| 040_std_function | std::function | `ClassTemplateSpecialization` | ⚠️ | ✅ |
| 041_functional_bind | std::bind | `CallExpr` | ❌ | ⚠️ |
| 042_exception_basic | 异常处理 | `CXXThrowExpr`, `CXXCatchStmt` | ✅ | ✅ |

#### 其他高级特性 (43-48)

| 示例 | 特性 | AST 节点 | v1 | v2 |
|------|------|----------|-----|-----|
| 043_namespace_nested | 嵌套命名空间 | `NamespaceDecl` (嵌套) | ✅ | ✅ |
| 044_enum_class | 强类型枚举 | `EnumDecl` (scoped) | ✅ | ✅ |
| 045_union_basic | 共用体 | `RecordDecl` (union) | ✅ | ✅ |
| 046_constexpr_basic | constexpr | `Expr` (constexpr) | ✅ | ✅ |
| 047_noexcept_basic | noexcept | `NoexceptSpec` | ✅ | ✅ |
| 048_summary | FFI 模式总结 | - | ✅ | ✅ |

**图例**：✅ 完全支持 ⚠️ 部分支持 ❌ 不支持

## 4. 核心设计

### 4.1 双模式解析架构

```rust
pub struct CppParser {
    index: clang::Index,
    compiler_args: Vec<String>,
}

impl CppParser {
    /// 模式 1：头文件解析（快速，获取声明）
    /// 适用于：类、函数、枚举、命名空间
    pub fn parse_header(&self, path: &Path) -> Result<ParseResult>;

    /// 模式 2：AST 编译捕获（完整，捕获模板实例化）
    /// 适用于：模板实例化、STL 容器
    pub fn parse_source(&self, path: &Path) -> Result<SourceParseResult>;
}

pub struct SourceParseResult {
    /// 模板实例化列表，如 std::vector<int>
    pub template_instantiations: Vec<TemplateInstantiation>,
    /// 所有引用的类型
    pub types: Vec<TypeInfo>,
    /// 所有函数
    pub functions: Vec<FunctionInfo>,
    /// 虚函数表信息
    pub vtable_info: Vec<VtableInfo>,
}

pub struct TemplateInstantiation {
    pub template_name: String,      // "vector"
    pub arguments: Vec<String>,     // ["int"]
    pub full_name: String,         // "std::vector<int>"
    pub location: SourceLocation,
}

pub struct VtableInfo {
    pub class_name: String,
    pub virtual_methods: Vec<VirtualMethod>,
}
```

### 4.2 libclang AST 编译捕获实现

```rust
use clang::{Index, CursorKind};

fn capture_instantiated_types(source_path: &Path) -> Result<Vec<TemplateInstantiation>> {
    let index = Index::new(false, true);

    // 编译源文件（触发模板实例化）
    let tu = index.parse_translation_unit(
        source_path,
        &["-std=c++17", "-I/usr/include/c++/11"],
    )?;

    let mut instantiations = Vec::new();
    let mut vtable_infos = Vec::new();

    // 遍历所有 AST 节点
    let cursor = tu.cursor();
    visit_children(&cursor, &mut |c| {
        match c.kind() {
            // 模板实例化，如 std::vector<int>
            CursorKind::ClassTemplateSpecialization => {
                if let Some(inst) = extract_template_instantiation(&c) {
                    instantiations.push(inst);
                }
            }
            // 虚函数
            CursorKind::CXXMethodDecl => {
                if c.is_virtual_method() {
                    // 处理虚函数
                }
            }
            // 抽象类
            CursorKind::CXXRecordDecl => {
                if c.is_abstract_class() {
                    vtable_infos.push(extract_vtable_info(&c));
                }
            }
            _ => {}
        }
    });

    Ok(instantiations)
}

fn extract_template_instantiation(cursor: &Cursor) -> Option<TemplateInstantiation> {
    let ty = cursor.cur_type()?;
    let spelling = ty.spelling()?;

    // 解析 "std::vector<int>" 获取模板名和参数
    if let Some((template_name, args)) = parse_template_type(&spelling) {
        Some(TemplateInstantiation {
            template_name,
            arguments: args,
            full_name: spelling,
            location: cursor.location()?,
        })
    } else {
        None
    }
}

fn parse_template_type(spelling: &str) -> Option<(String, Vec<String>)> {
    // "std::vector<int>" -> ("vector", ["int"])
    // "std::map<std::string, int>" -> ("map", ["std::string", "int"])
    // ...
}
```

### 4.3 模板实例化处理

```rust
impl TemplateInstantiation {
    /// 判断是否是 STL 容器
    pub fn is_stl_container(&self) -> bool {
        matches!(self.template_name.as_str(),
            "vector" | "list" | "deque" |
            "map" | "set" | "unordered_map" | "unordered_set" |
            "string" | "basic_string"
        )
    }

    /// 判断是否是智能指针
    pub fn is_smart_pointer(&self) -> bool {
        matches!(self.template_name.as_str(),
            "unique_ptr" | "shared_ptr" | "weak_ptr" |
            "auto_ptr" | "scoped_ptr"
        )
    }

    /// 转换为 Rust FFI 类型
    pub fn to_rust_ffi_type(&self) -> RustType {
        if self.is_stl_container() {
            self.to_hicc_std_type()
        } else if self.is_smart_pointer() {
            self.to_hicc_smart_ptr_type()
        } else {
            self.to_opaque_ptr_type()
        }
    }

    /// 生成 hicc-std 包装类型
    fn to_hicc_std_type(&self) -> RustType {
        match self.template_name.as_str() {
            "vector" => {
                let inner = map_cpp_type_to_rust(&self.arguments[0]);
                RustType::HiccStd(format!("hicc_std::vector<{}>", inner))
            }
            "map" => {
                let key = map_cpp_type_to_rust(&self.arguments[0]);
                let val = map_cpp_type_to_rust(&self.arguments[1]);
                RustType::HiccStd(format!("hicc_std::map<{}, {}>", key, val))
            }
            "string" => {
                RustType::HiccStd("hicc_std::string".to_string())
            }
            _ => self.to_opaque_ptr_type()
        }
    }

    /// 生成 opaque pointer 类型
    fn to_opaque_ptr_type(&self) -> RustType {
        RustType::OpaquePtr(format!(
            "*mut std::ffi::c_void /* {} */",
            self.full_name
        ))
    }
}
```

### 4.4 虚函数表映射

```rust
pub struct VirtualMethod {
    pub name: String,
    pub signature: String,        // "double () const"
    pub is_pure_virtual: bool,
    pub index: usize,            // vtable 索引
}

pub struct VtableInfo {
    pub class_name: String,
    pub full_name: String,        // "foo::bar::AbstractShape"
    pub virtual_methods: Vec<VirtualMethod>,
    pub bases: Vec<BaseInfo>,    // 基类信息
}

impl VtableInfo {
    /// 转换为 Rust trait
    pub fn to_rust_trait(&self) -> String {
        let methods: Vec<String> = self.virtual_methods
            .iter()
            .map(|m| {
                let args = extract_args(&m.signature);
                let ret = extract_return_type(&m.signature);
                if m.is_pure_virtual {
                    format!("fn {}({}) -> {} {{ unimplemented!() }}", m.name, args, ret)
                } else {
                    format!("fn {}({}) -> {};", m.name, args, ret)
                }
            })
            .collect();

        format!(
            "trait {} {{\n    {}\n}}",
            self.class_name,
            methods.join("\n    ")
        )
    }
}
```

### 4.5 C++ 类型到 Rust 类型映射

```rust
fn map_cpp_type_to_rust(cpp_type: &str) -> String {
    match cpp_type {
        // 基本类型
        "int" => "i32".to_string(),
        "unsigned int" | "uint32_t" => "u32".to_string(),
        "char" => "i8".to_string(),
        "unsigned char" => "u8".to_string(),
        "short" => "i16".to_string(),
        "unsigned short" => "u16".to_string(),
        "long" => "i64".to_string(),
        "unsigned long" => "u64".to_string(),
        "size_t" => "usize".to_string(),
        "void" => "()".to_string(),
        "bool" => "bool".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),

        // 指针类型
        s if s.ends_with("* const") => {
            let inner = &s[..s.len()-7].trim();
            format!("*const {}", map_cpp_type_to_rust(inner))
        }
        s if s.ends_with("*") => {
            let inner = &s[..s.len()-1].trim();
            format!("*mut {}", map_cpp_type_to_rust(inner))
        }

        // 引用类型
        s if s.ends_with("&") && !s.contains("const") => {
            let inner = &s[..s.len()-1].trim();
            format!("&mut {}", map_cpp_type_to_rust(inner))
        }
        s if s.ends_with("const &") => {
            let inner = &s[..s.len()-7].trim();
            format!("&{}", map_cpp_type_to_rust(inner))
        }

        // 模板类型
        s if s.starts_with("std::vector<") => {
            let inner = extract_template_arg(s, "std::vector");
            format!("Vec<{}>", map_cpp_type_to_rust(inner))
        }
        s if s.starts_with("std::map<") => {
            let (key, val) = extract_two_args(s, "std::map");
            format!("std::collections::HashMap<{}, {}>",
                map_cpp_type_to_rust(key),
                map_cpp_type_to_rust(val)
            )
        }
        s if s.starts_with("std::string") => {
            "String".to_string()
        }

        _ => format!("*mut std::ffi::c_void /* {} */", cpp_type)
    }
}
```

## 5. Rust FFI 代码生成

### 5.1 普通类生成（复用 v1）

```rust
// 输入：foo::bar::ConfigManager
hicc::cpp! {
    namespace foo { namespace bar {
    class ConfigManager { /* ... */ };
    }}
}

type ConfigManager = *mut std::ffi::c_void;

#[link(name = "libname")]
unsafe extern "C" {
    fn config_manager_new() -> ConfigManager;
    fn config_manager_delete(p: ConfigManager);
}
```

### 5.2 模板实例化生成（v2 新增）

```rust
// 输入：std::vector<int>
hicc::cpp! {
    #include <vector>

    typedef std::vector<int> IntVector;
}

// 生成 hicc-std 包装
hicc::import_lib! {
    #![link_name = "mystl"]

    // std::vector<int>
    #[cpp(func = "std::vector<int>* std_vector_int_new()")]
    fn std_vector_int_new() -> hicc_std::vector<hicc::Pod<i32>>;

    #[cpp(func = "void std_vector_int_delete(std::vector<int>*)")]
    unsafe fn std_vector_int_delete(v: &mut hicc_std::vector<hicc::Pod<i32>>);
}
```

### 5.3 虚函数表生成（v2 新增）

```rust
// 输入：AbstractShape (抽象类)
hicc::cpp! {
    class AbstractShape {
    public:
        virtual ~AbstractShape() = default;
        virtual double area() const = 0;
        virtual const char* getName() const = 0;
    };

    class Circle : public AbstractShape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
        const char* getName() const override;
    };
}

// 生成 Rust trait
hicc::import_class! {
    #[interface]
    #[cpp(class = "AbstractShape")]
    class AbstractShape {
        #[cpp(method = "virtual double area() const = 0")]
        fn area(&self) -> f64;

        #[cpp(method = "virtual const char* getName() const = 0")]
        fn get_name(&self) -> *const i8;
    }
}
```

### 5.4 完整示例

**C++ 源码** (`main.cpp`)：
```cpp
#include <vector>
#include <string>
#include <map>

class AbstractShape {
public:
    virtual ~AbstractShape() = default;
    virtual double area() const = 0;
};

class Circle : public AbstractShape {
    double radius;
public:
    Circle(double r) : radius(r) {}
    double area() const override { return 3.14159 * radius * radius; }
};

int main() {
    std::vector<Circle*> shapes;
    shapes.push_back(new Circle(1.0));
    return 0;
}
```

**AST 编译捕获结果**：
```rust
TemplateInstantiation { template_name: "vector", arguments: ["Circle *"] }
ClassTemplateSpecialization { name: "vector", args: ["Circle *"] }
VtableInfo { class_name: "AbstractShape", virtual_methods: [area] }
```

**生成的 Rust FFI**：
```rust
hicc::cpp! {
    #include <vector>

    class AbstractShape {
    public:
        virtual ~AbstractShape() = default;
        virtual double area() const = 0;
    };

    class Circle : public AbstractShape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
    };
}

// 抽象类接口
hicc::import_class! {
    #[interface]
    #[cpp(class = "AbstractShape")]
    class AbstractShape {
        #[cpp(method = "virtual double area() const = 0")]
        fn area(&self) -> f64;
    }
}

// 模板实例化
hicc::import_lib! {
    #![link_name = "shapes"]

    #[cpp(func = "std::vector<Circle*>* circle_vector_new()")]
    fn circle_vector_new() -> hicc_std::vector<*mut AbstractShape>;

    #[cpp(func = "void circle_vector_delete(std::vector<Circle*>*)")]
    unsafe fn circle_vector_delete(v: &mut hicc_std::vector<*mut AbstractShape>);
}
```

## 6. 实现计划

### 6.1 阶段划分

| 阶段 | 内容 | 优先级 | 覆盖示例 |
|------|------|--------|----------|
| **Phase A** | AST 编译捕获基础架构 | P0 | 所有 v2 基础 |
| **Phase B** | 模板实例化提取 | P0 | 025, 027, 034-038 |
| **Phase C** | STL 容器识别与 hicc-std 映射 | P1 | 034, 035, 036 |
| **Phase D** | 虚函数表映射 | P1 | 015, 016, 017, 018 |
| **Phase E** | 智能指针支持 | P2 | 029, 030 |
| **Phase F** | Lambda/std::function 支持 | P2 | 039, 040 |
| **Phase G** | 集成测试 + 48 示例验证 | P1 | 全部 |

### 6.2 Phase A: AST 编译捕获基础架构

**目标**：实现 libclang 编译源文件并遍历 AST

**关键代码**：
```rust
pub struct AstCompiler {
    index: clang::Index,
    args: Vec<String>,
}

impl AstCompiler {
    pub fn new() -> Result<Self> {
        let index = Index::new(false, true);
        Ok(Self { index, args: vec![] })
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn compile(&self, path: &Path) -> Result<TranslationUnit> {
        self.index
            .parse_translation_unit(path, &self.args)
            .map_err(|e| anyhow!("failed to parse: {}", e))
    }
}

impl TranslationUnit {
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(Cursor) -> bool,
    {
        self.cursor.visit_children(visitor);
    }
}
```

### 6.3 Phase B: 模板实例化提取

**目标**：识别 ClassTemplateSpecialization 节点

**关键代码**：
```rust
fn visit_cursor(cursor: &Cursor, results: &mut Vec<TemplateInstantiation>) {
    match cursor.kind() {
        CursorKind::ClassTemplateSpecialization => {
            if let Some(inst) = parse_template_spec(cursor) {
                results.push(inst);
            }
        }
        _ => {}
    }
    cursor.visit_children(&mut |c| {
        visit_cursor(&c, results);
        true
    });
}

fn parse_template_spec(cursor: &Cursor) -> Option<TemplateInstantiation> {
    let ty = cursor.cur_type()?;
    let spelling = ty.spelling()?;

    // "std::vector<int>" -> template_name="vector", args=["int"]
    parse_template_name(&spelling)
}
```

### 6.4 Phase C: STL 容器识别

**目标**：自动识别 STL 容器并生成 hicc-std 包装

```rust
const STL_CONTAINERS: &[&str] = &[
    "vector", "list", "deque",
    "map", "multimap", "set", "multiset",
    "unordered_map", "unordered_multimap",
    "unordered_set", "unordered_multiset",
    "string", "basic_string",
];

fn is_stl_container(name: &str) -> bool {
    STL_CONTAINERS.iter().any(|&c| name == c)
}
```

### 6.5 Phase D: 虚函数表映射

**目标**：支持抽象类到 Rust trait

```rust
fn extract_vtable_info(cursor: &Cursor) -> Option<VtableInfo> {
    if !cursor.is_abstract_class() {
        return None;
    }

    let mut virtual_methods = Vec::new();
    let mut index = 0;

    for child in cursor.children() {
        if child.kind() == CursorKind::CXXMethodDecl && child.is_virtual_method() {
            virtual_methods.push(VirtualMethod {
                name: child.spelling()?,
                signature: child.cur_type()?.spelling()?,
                is_pure_virtual: child.is_pure_virtual_method(),
                index,
            });
            index += 1;
        }
    }

    Some(VtableInfo {
        class_name: cursor.spelling()?,
        full_name: cursor.cur_type()?.spelling()?,
        virtual_methods,
        bases: extract_bases(cursor),
    })
}
```

## 7. 技术依赖

### 7.1 Rust 依赖

```toml
[dependencies]
clang = "0.1"          # libclang 绑定
clap = "4"              # CLI
anyhow = "1"            # 错误处理
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
cc = "1"                # C++ 编译器调用
```

### 7.2 系统依赖

```bash
# Ubuntu/Debian
apt-get install clang-18 libclang-18-dev

# 需要 C++ 标准库头文件
apt-get install libstdc++-12-dev
```

## 8. 测试计划

### 8.1 单元测试

| 测试 | 内容 |
|------|------|
| `ast_compiler_tests` | libclang 编译和 AST 遍历 |
| `template_parsing_tests` | 模板实例化解析 (034-038) |
| `stl_container_tests` | STL 容器识别与映射 |
| `vtable_tests` | 虚函数表解析 (015-018) |
| `type_mapping_tests` | C++ 到 Rust 类型映射 |

### 8.2 集成测试矩阵

```
        | 头文件解析 | AST编译捕获 | STL容器 | 虚函数表 |
--------|-----------|------------|---------|----------|
001-005 |     ✅     |     -      |    -    |    -     |
006-012 |     ✅     |     -      |    -    |    -     |
013-018 |     ✅     |     -      |    -    |    ⚠️     |
024-028 |     ⚠️     |     ✅      |    -    |    -     |
029-033 |     ✅     |     -      |    -    |    -     |
034-038 |     ⚠️     |     ✅      |    ✅    |    -     |
039-042 |     ❌     |     ⚠️      |    ⚠️    |    -     |
043-047 |     ✅     |     -      |    -    |    -     |
```

### 8.3 验收标准

1. **模板实例化**：34-038 所有 STL 容器示例能自动捕获
2. **虚函数表**：015-018 继承相关示例能生成 Rust trait
3. **编译通过**：所有生成的 rust_hicc 项目可以 `cargo build`
4. **运行正确**：生成的代码运行结果与手动编写一致

## 9. 已知限制

### 9.1 技术限制

1. **仅捕获使用的实例化**：
   - 只有代码中实际使用的 `std::vector<int>` 才会被捕获
   - 未使用的模板实例化不会被生成

2. **运算符重载**：需要语义分析，复杂度高，v2 仍不支持

3. **友元函数/typeid**：需要更深入的语义分析

### 9.2 解决方案

1. **手动补充**：
   ```rust
   // 生成 TODO 注释
   hicc::cpp! {
       // TODO: 手动添加未捕获的模板实例化
       typedef std::vector<std::string> StringVector;
   }
   ```

2. **可选的穷举模式**：
   ```bash
   # 尝试推导所有可能的实例化
   cpp2rust-ffi --input ./cpp --output ./rust --exhaustive-templates
   ```

## 10. 与 v1 的关系

### 10.1 兼容性

v2 完全兼容 v1 的功能：
- 头文件解析模式继续支持
- 所有 v1 生成的代码格式保持一致
- CLI 接口向下兼容

### 10.2 增量开发

```
v1 (当前)
  └── parse_header()  # 头文件解析

v2 (计划)
  └── parse_header()  # v1 复用
  └── parse_source()  # 新增 AST 编译捕获
  └── generate_vtable()  # 新增虚函数表映射
```

## 11. 总结

v2 计划的核心改进：

1. **新增 AST 编译捕获**：通过 libclang 编译源文件，捕获模板实例化
2. **STL 容器支持**：自动识别并生成 hicc-std 包装 (034-038)
3. **虚函数表映射**：支持抽象类到 Rust trait (015-018)
4. **完全兼容 v1**：所有 v1 功能继续工作

**特性覆盖对比**：

| | v1 | v2 |
|---|-----|-----|
| **原理** | 头文件解析 | 头文件解析 + AST 编译 |
| **模板实例化** | ❌ | ✅ `std::vector<int>` |
| **STL 容器** | ❌ | ✅ hicc-std 包装 |
| **虚函数表** | ⚠️ | ✅ Rust trait |
| **Lambda** | ❌ | ⚠️ |
| **运算符重载** | ❌ | ❌ |
