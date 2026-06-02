use crate::error::Result;
use std::path::PathBuf;

/// Abstraction over file-selection so tests can inject a fake implementation.
pub trait FileSelector {
    /// Given a slice of candidate `.cpp2rust` file paths, return the subset the
    /// user wants to include in this feature.
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>>;
}

/// Interactive multi-select backed by `dialoguer`.
///
/// When stdin is not a terminal (CI, pipes, tests) this automatically selects
/// all candidates so the workflow is never blocked waiting for user input.
pub struct InteractiveSelector;

impl FileSelector for InteractiveSelector {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        if candidates.is_empty() {
            println!("No .cpp2rust files found – nothing to select.");
            return Ok(vec![]);
        }

        if is_non_interactive() {
            println!(
                "Non-interactive terminal: selecting all {} file(s) automatically.",
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

/// Selector that selects all candidates without user interaction. Used in tests.
#[allow(dead_code)]
pub struct SelectAll;

impl FileSelector for SelectAll {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(candidates.to_vec())
    }
}

/// Selector that selects no candidates. Used in tests.
#[allow(dead_code)]
pub struct SelectNone;

impl FileSelector for SelectNone {
    fn select(&self, _candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }
}

/// Selector backed by a predicate closure. Used in tests.
#[allow(dead_code)]
pub struct PredicateSelector<F>(pub F)
where
    F: Fn(&PathBuf) -> bool;

impl<F: Fn(&PathBuf) -> bool> FileSelector for PredicateSelector<F> {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(candidates.iter().filter(|p| (self.0)(p)).cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
