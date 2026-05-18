use crate::error::Result;
use std::path::PathBuf;

/// Abstraction over middleware-file selection so tests can inject a fake implementation.
pub trait FileSelector {
    /// Given a slice of candidate middleware paths captured by the LD_PRELOAD hook,
    /// return the subset the user wants to process in this feature.
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>>;
}

/// Selects all candidates automatically, enforcing the full-capture principle.
///
/// Previously this presented an interactive MultiSelect in terminal mode, allowing
/// users to deselect individual middleware files.  The full-capture principle
/// requires all captured translation units to be processed, so the interactive
/// prompt has been removed.  All candidates are always selected.
pub struct InteractiveSelector;

impl FileSelector for InteractiveSelector {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        if candidates.is_empty() {
            println!("No captured *.cpp2rust middleware files found – nothing to select.");
            return Ok(vec![]);
        }

        println!(
            "Full-capture mode: selecting all {} file(s) automatically.",
            candidates.len()
        );
        Ok(candidates.to_vec())
    }
}

/// Selector that selects all candidates without user interaction.  Used in tests.
#[allow(dead_code)]
pub struct SelectAll;

impl FileSelector for SelectAll {
    fn select(&self, candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(candidates.to_vec())
    }
}

/// Selector that selects no candidates.  Used in tests.
#[allow(dead_code)]
pub struct SelectNone;

impl FileSelector for SelectNone {
    fn select(&self, _candidates: &[PathBuf]) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }
}

/// Selector backed by a predicate closure.  Used in tests.
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
        let paths = make_paths(&["a.hpp.cpp2rust", "b.cc.cpp2rust"]);
        let result = SelectAll.select(&paths).unwrap();
        assert_eq!(result, paths);
    }

    #[test]
    fn select_none_returns_empty() {
        let paths = make_paths(&["a.hpp.cpp2rust", "b.cc.cpp2rust"]);
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
        let paths = make_paths(&[
            "foo/a.hpp.cpp2rust",
            "bar/b.cc.cpp2rust",
            "foo/c.hpp.cpp2rust",
        ]);
        let sel = PredicateSelector(|p: &PathBuf| p.to_str().map_or(false, |s| s.contains("foo")));
        let result = sel.select(&paths).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn interactive_selector_empty_input() {
        let result = InteractiveSelector.select(&[]).unwrap();
        assert!(result.is_empty());
    }
}
