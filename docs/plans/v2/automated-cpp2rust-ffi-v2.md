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

**根本原因**：头文件解析只是语法分析，模板实例化发生在**编译器的语义分析阶段**。

### 1.2 v2 改进目标

1. **支持模板实例化捕获**：通过 AST 编译捕获，获取 `std::vector<int>` 等实例化类型
2. **支持 STL 容器**：自动识别并生成 hicc-std 包装
3. **保持 v1 特性**：所有 v1 功能继续支持

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
│   ├── parser/                    # C++ 解析
│   │   ├── mod.rs
│   │   ├── header_parser.rs       # 头文件解析 (v1 复用)
│   │   └── ast_compiler.rs       # AST 编译捕获 (新增)
│   ├── generator/                 # Rust 代码生成
│   │   ├── mod.rs
│   │   ├── class_generator.rs
│   │   ├── template_generator.rs  # 模板实例化生成 (新增)
│   │   └── project_generator.rs
│   └── template/                 # 项目模板
│       └── ...
├── Cargo.toml
└── README.md
```

## 3. 核心设计

### 3.1 双模式解析架构

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
}

pub struct TemplateInstantiation {
    pub template_name: String,      // "vector"
    pub arguments: Vec<String>,     // ["int"]
    pub full_name: String,         // "std::vector<int>"
    pub location: SourceLocation,
}
```

### 3.2 libclang AST 编译捕获实现

```rust
use clang::{Index, CursorKind};

fn capture_instantiated_types(source_path: &Path) -> Result<Vec<TemplateInstantiation>> {
    let index = Index::new(false, true); // createPCH=false, completeDiagnostics=true

    // 编译源文件（触发模板实例化）
    let tu = index.parse_translation_unit(
        source_path,
        &["-std=c++17", "-I/usr/include/c++/11"],
    )?;

    let mut instantiations = Vec::new();

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
            // 模板引用
            CursorKind::TemplateRef => {
                // 处理模板引用
            }
            // 类型引用
            CursorKind::TypeRef => {
                // 处理类型引用
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
```

### 3.3 模板实例化处理

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

### 3.4 C++ 类型到 Rust 类型映射

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
        s if s.ends_with("*") && s.contains("const") => {
            let inner = &s[..s.len()-1].trim();
            format!("*const {}", map_cpp_type_to_rust(inner.trim()))
        }
        s if s.ends_with("*") => {
            let inner = &s[..s.len()-1].trim();
            format!("*mut {}", map_cpp_type_to_rust(inner.trim()))
        }

        // 模板类型
        s if s.starts_with("std::vector<") => {
            // 解析并递归映射
            let inner = extract_template_arg(s, "std::vector");
            format!("Vec<{}>", map_cpp_type_to_rust(inner))
        }

        _ => format!("*mut std::ffi::c_void /* {} */", cpp_type)
    }
}
```

## 4. Rust FFI 代码生成

### 4.1 普通类生成（复用 v1）

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

### 4.2 模板实例化生成（v2 新增）

```rust
// 输入：std::vector<int>
hicc::cpp! {
    #include <vector>

    typedef std::vector<int> IntVector;
}

// 生成 hicc-std 包装
hicc::import_lib! {
    #![link_name = "mystl"]
    class std::vector<int> = hicc_std::vector<hicc::Pod<i32>>;
}
```

### 4.3 完整示例

**C++ 源码** (`main.cpp`)：
```cpp
#include <vector>
#include <string>
#include <map>

int main() {
    std::vector<int> int_vec = {1, 2, 3};
    std::map<std::string, int> str_int_map = {{"key", 42}};
    return 0;
}
```

**AST 编译捕获结果**：
```rust
TemplateInstantiation { template_name: "vector", arguments: ["int"] }
TemplateInstantiation { template_name: "map", arguments: ["std::string", "int"] }
```

**生成的 Rust FFI**：
```rust
hicc::cpp! {
    #include <vector>
    #include <string>
    #include <map>

    typedef std::vector<int> IntVector;
    typedef std::map<std::string, int> StringIntMap;
}

hicc::import_lib! {
    #![link_name = "mystl"]

    // std::vector<int>
    #[cpp(func = "std::vector<int>* std_vector_int_new()")]
    fn std_vector_int_new() -> hicc_std::vector<hicc::Pod<i32>>;

    #[cpp(func = "void std_vector_int_delete(std::vector<int>*)")]
    unsafe fn std_vector_int_delete(v: &mut hicc_std::vector<hicc::Pod<i32>>);

    // std::map<std::string, int>
    #[cpp(func = "std::map<std::string, int>* std_map_string_int_new()")]
    fn std_map_string_int_new() -> hicc_std::map<hicc_std::string, i32>;

    #[cpp(func = "void std_map_string_int_delete(std::map<std::string, int>*)")]
    unsafe fn std_map_string_int_delete(m: &mut hicc_std::map<hicc_std::string, i32>);
}
```

## 5. 实现计划

### 5.1 阶段划分

| 阶段 | 内容 | 优先级 |
|------|------|--------|
| **Phase A** | AST 编译捕获基础架构 | P0 |
| **Phase B** | 模板实例化提取 | P0 |
| **Phase C** | STL 容器识别与 hicc-std 映射 | P1 |
| **Phase D** | 智能指针支持 | P2 |
| **Phase E** | 集成测试 + 48 示例验证 | P1 |

### 5.2 Phase A: AST 编译捕获基础架构

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

pub struct TranslationUnit {
    cursor: Cursor,
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

### 5.3 Phase B: 模板实例化提取

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

### 5.4 Phase C: STL 容器识别

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

## 6. 技术依赖

### 6.1 Rust 依赖

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

### 6.2 系统依赖

```bash
# Ubuntu/Debian
apt-get install clang-18 libclang-18-dev

# 需要 C++ 标准库头文件
apt-get install libstdc++-12-dev
```

## 7. 测试计划

### 7.1 单元测试

| 测试 | 内容 |
|------|------|
| `ast_compiler_tests` | libclang 编译和 AST 遍历 |
| `template_parsing_tests` | 模板实例化解析 |
| `type_mapping_tests` | C++ 到 Rust 类型映射 |
| `hicc_generation_tests` | hicc 宏生成 |

### 7.2 集成测试

```bash
# 测试用例
for dir in examples/0*/; do
    if [ -d "$dir/cpp" ]; then
        echo "Testing $dir"
        cpp2rust-ffi -i "$dir/cpp" -o /tmp/out --capture-ast
        diff -r "$dir/rust_hicc" /tmp/out || echo "DIFF in $dir"
    fi
done
```

### 7.3 模板专用测试

```bash
# 创建模板测试用例
mkdir -p examples/049_template_demo
cat > examples/049_template_demo/cpp/template_demo.h << 'EOF'
#include <vector>
#include <map>

template<typename T>
T max_value(const std::vector<T>& vec) { /* ... */ }

std::vector<int>* create_int_vector();
EOF

cpp2rust-ffi -i examples/049_template_demo/cpp -o /tmp/out
# 验证 std::vector<int> 被正确捕获
grep -q "std::vector<int>" /tmp/out/src/main.rs
```

## 8. 已知限制

### 8.1 技术限制

1. **仅捕获使用的实例化**：
   - 只有代码中实际使用的 `std::vector<int>` 才会被捕获
   - 未使用的模板实例化不会被生成

2. **模板偏特化**：v2 仍不支持模板偏特化

3. **运算符重载**：需要语义分析，复杂度高

### 8.2 解决方案

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

## 9. 与 v1 的关系

### 9.1 兼容性

v2 完全兼容 v1 的功能：
- 头文件解析模式继续支持
- 所有 v1 生成的代码格式保持一致
- CLI 接口向下兼容

### 9.2 增量开发

```
v1 (当前)
  └── parse_header()  # 头文件解析

v2 (计划)
  └── parse_header()  # v1 复用
  └── parse_source()  # 新增 AST 编译捕获
```

## 10. 总结

v2 计划的核心改进：

1. **新增 AST 编译捕获**：通过 libclang 编译源文件，捕获模板实例化
2. **STL 容器支持**：自动识别并生成 hicc-std 包装
3. **完全兼容 v1**：所有 v1 功能继续工作

**关键区别**：
- v1：头文件解析 → 获取声明
- v2：头文件解析 + AST 编译 → 获取声明 + 实例化
