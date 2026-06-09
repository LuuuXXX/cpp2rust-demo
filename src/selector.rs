use crate::error::Result;
use std::path::PathBuf;

/// 文件选择的抽象接口，便于测试时注入假实现。
pub trait FileSelector {
    /// 给定候选 `.cpp2rust` 文件路径切片，返回用户希望包含在本特性中的子集。
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>>;
}

/// 基于 `dialoguer` 的交互式多选。
///
/// 当 stdin 不是终端（CI、管道、测试等）时，自动选择全部候选文件，
/// 避免工作流因等待用户输入而阻塞。
pub struct InteractiveSelector;

impl FileSelector for InteractiveSelector {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        if candidates.is_empty() {
            println!("未找到 .cpp2rust 文件，没有可选内容。");
            return Ok(vec![]);
        }

        if is_non_interactive() {
            println!(
                "非交互式终端：自动选择全部 {} 个文件。",
                candidates.len()
            );
            return Ok(candidates.to_vec());
        }

        use dialoguer::{theme::ColorfulTheme, MultiSelect};

        let items: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Select files to include in this feature (space to toggle, enter to confirm)",
            )
            .items(&items)
            .defaults(&vec![true; items.len()])
            .interact()
            .map_err(|e| anyhow::anyhow!("interactive selection failed: {}", e))?;

        Ok(selections
            .into_iter()
            .map(|i| candidates[i].clone())
            .collect())
    }
}

/// 判断当前运行环境是否为非交互式终端（CI、管道、测试等）。
///
/// 当 stdin 不是 TTY 时返回 `true`，此时应跳过用户交互自动选择全部文件。
fn is_non_interactive() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 无需用户交互、直接选择全部候选文件的选择器。
    struct SelectAll;
    impl FileSelector for SelectAll {
        fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
            Ok(candidates.to_vec())
        }
    }

    /// 不选择任何候选文件的选择器。
    struct SelectNone;
    impl FileSelector for SelectNone {
        fn select(&self, _candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
            Ok(vec![])
        }
    }

    /// 基于谓词闭包的选择器。
    struct PredicateSelector<F: Fn(&PathBuf) -> bool>(F);
    impl<F: Fn(&PathBuf) -> bool> FileSelector for PredicateSelector<F> {
        fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
            Ok(candidates.iter().filter(|p| (self.0)(p)).cloned().collect())
        }
    }

    fn make_paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn select_all_returns_all() {
        let paths = make_paths(&["a.cpp2rust", "b.cpp2rust"]);
        let result = SelectAll.select(&paths).unwrap();
        assert_eq!(result, paths);
    }

    #[test]
    fn select_none_returns_empty() {
        let paths = make_paths(&["a.cpp2rust", "b.cpp2rust"]);
        let result = SelectNone.select(&paths).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn select_all_empty_input() {
        let result = SelectAll.select(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn predicate_selector_filters() {
        let paths = make_paths(&["foo/a.cpp2rust", "bar/b.cpp2rust", "foo/c.cpp2rust"]);
        let sel = PredicateSelector(|p: &PathBuf| p.to_str().is_some_and(|s| s.contains("foo")));
        let result = sel.select(&paths).unwrap();
        assert_eq!(result.len(), 2);
    }
}
