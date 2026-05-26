use crate::types::*;
use std::collections::HashMap;

/// 模板实例化追踪器
/// 收集 AST 中实际出现的模板实例化版本
pub struct InstantiationTracker {
    /// 模板名 → 实例化参数列表
    pub instantiations: HashMap<String, Vec<Vec<String>>>,
}

impl InstantiationTracker {
    pub fn new() -> Self {
        Self {
            instantiations: HashMap::new(),
        }
    }

    /// 从 CppAst 中收集模板实例化信息
    pub fn collect_from_ast(&mut self, ast: &CppAst) {
        for class in &ast.classes {
            if class.is_template_specialization && !class.template_args.is_empty() {
                self.instantiations
                    .entry(class.name.clone())
                    .or_default()
                    .push(class.template_args.clone());
            }
        }
    }

    /// 获取某个模板的所有实例化参数
    pub fn get_instantiations(&self, template_name: &str) -> Vec<&Vec<String>> {
        self.instantiations
            .get(template_name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for InstantiationTracker {
    fn default() -> Self {
        Self::new()
    }
}
