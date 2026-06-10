//! 纯数据结构 — API 接口清单、报告数据、FeatureLayout 目录布局
//!
//! 所有结构体均不包含 I/O 操作，文件读写逻辑见 `io.rs`。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─────────────────────────────────────────────
//  API 接口清单数据结构
// ─────────────────────────────────────────────

/// merge 阶段生成的 API 接口清单（序列化为 `meta/api-manifest.md`）。
/// 用于支持 C++ → Rust API 对账：逐条记录 C++ 签名与对应 Rust 绑定。
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiManifest {
    /// feature 名称
    pub feature: String,
    /// 类绑定列表（按首次出现顺序排列）
    pub classes: Vec<ApiClassEntry>,
    /// 独立函数绑定列表
    pub functions: Vec<ApiFunctionEntry>,
    /// 模板特化分组：`(base_template_name, [specialization_names…])`
    /// 例如 `("Stack", ["Stack<int>", "Stack<double>"])`。
    #[serde(default)]
    pub template_groups: Vec<(String, Vec<String>)>,
}

/// 单个类的绑定信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiClassEntry {
    /// C++ 类名
    pub name: String,
    /// 类属性行（如 `#[cpp(class = "Foo")]` 或 `#[interface]`）
    pub class_attr: String,
    /// 方法绑定列表
    pub methods: Vec<ApiMethodEntry>,
}

/// 单个类方法的绑定信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiMethodEntry {
    /// C++ 方法签名（如 `int get() const`）
    pub cpp_sig: String,
    /// Rust 方法签名（如 `fn get(&self) -> i32;`）
    pub rust_sig: String,
    /// 是否含降级标记（`cpp2rust-todo`）
    pub is_degraded: bool,
}

/// 独立函数绑定信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiFunctionEntry {
    /// C++ 函数签名（如 `Foo* foo_new(int v)`）
    pub cpp_sig: String,
    /// Rust 函数签名（如 `fn foo_new(v: i32) -> *mut Foo;`）
    pub rust_sig: String,
    /// 是否含降级标记（`cpp2rust-todo`）
    pub is_degraded: bool,
}

// ─────────────────────────────────────────────
//  报告数据结构
// ─────────────────────────────────────────────

/// init 阶段单个编译单元的统计信息。
pub struct InitUnitStat {
    /// `.cpp2rust` 文件路径（用于显示）
    pub cpp2rust_path: String,
    /// 派生的 Rust 模块路径（如 `utils/foo`）
    pub unit_path: String,
    /// 解析到的 C++ 类数量
    pub class_count: usize,
    /// 解析到的 C++ 函数数量
    pub fn_count: usize,
    /// 解析到的 C++ 枚举数量
    pub enum_count: usize,
    /// 处理该文件耗时（毫秒）
    pub elapsed_ms: u128,
}

/// init 阶段报告所需的完整数据。
pub struct InitReportData<'a> {
    pub feature: &'a str,
    pub build_cmd: &'a str,
    pub captured_count: usize,
    pub selected_count: usize,
    pub units: &'a [InitUnitStat],
    /// 降级标签列表：`(tag, [(unit_path, count)])`，按 tag 名排序；
    /// 每个元素包含该 tag 在各编译单元中的出现次数，用于精确定位。
    pub degraded_tags: &'a [(String, Vec<(String, usize)>)],
}

/// merge 阶段报告所需的完整数据。
pub struct MergeReportData<'a> {
    pub feature: &'a str,
    pub unit_count: usize,
    /// 合并时发现的冲突描述列表
    pub conflicts: &'a [String],
    /// 合并后生成的 .rs 文件总数
    pub rs_file_count: usize,
    /// 包含 `hicc::import_lib!` 块的文件数
    pub import_lib_files: usize,
    /// 包含 `hicc::import_class!` 块的文件数
    pub import_class_files: usize,
    /// `#[cpp(func = "...")]` 绑定函数总数
    pub fn_binding_count: usize,
    /// 降级标记总数（`cpp2rust-todo`）
    pub todo_count: usize,
    /// link_name 含路径分隔符的异常数量
    pub bad_link_name_count: usize,
}

// ─────────────────────────────────────────────
//  目录布局
// ─────────────────────────────────────────────

/// `.cpp2rust/<feature>/` 目录结构描述。
pub struct FeatureLayout {
    pub project_root: PathBuf,
    /// `.cpp2rust/<feature>/`
    pub feature_root: PathBuf,
    /// `.cpp2rust/<feature>/c/`（预处理文件）
    pub c_dir: PathBuf,
    /// `.cpp2rust/<feature>/rust/`（生成的 Rust 项目）
    pub rust_dir: PathBuf,
    /// `.cpp2rust/<feature>/meta/`（元数据：报告、清单等）
    pub meta_dir: PathBuf,
}
