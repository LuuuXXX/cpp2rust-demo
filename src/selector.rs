use crate::error::Result;
use dialoguer::MultiSelect;
use serde::{Deserialize, Serialize};
use std::io::{stderr, stdin, stdout, IsTerminal};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedFile {
    pub source_rel: PathBuf,
}

pub fn select_files(paths: &[PathBuf]) -> Result<Vec<SelectedFile>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    if std::env::var_os("CPP2RUST_AUTO_SELECT").is_some()
        || !stdin().is_terminal()
        || !stdout().is_terminal()
        || !stderr().is_terminal()
    {
        return Ok(paths
            .iter()
            .cloned()
            .map(|source_rel| SelectedFile { source_rel })
            .collect());
    }

    let items = paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let selected = MultiSelect::new()
        .with_prompt("Select translation units to generate")
        .items(&items)
        .defaults(&vec![true; items.len()])
        .interact()?;
    Ok(selected
        .into_iter()
        .map(|index| SelectedFile {
            source_rel: paths[index].clone(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selects_non_interactive_files() {
        std::env::set_var("CPP2RUST_AUTO_SELECT", "1");
        let selected = select_files(&[PathBuf::from("a.cpp"), PathBuf::from("b.cpp")]).unwrap();
        std::env::remove_var("CPP2RUST_AUTO_SELECT");
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn empty_input_returns_empty_selection() {
        let selected = select_files(&[]).unwrap();
        assert!(selected.is_empty());
    }

    #[test]
    fn selection_preserves_order() {
        std::env::set_var("CPP2RUST_AUTO_SELECT", "1");
        let selected = select_files(&[PathBuf::from("b.cpp"), PathBuf::from("a.cpp")]).unwrap();
        std::env::remove_var("CPP2RUST_AUTO_SELECT");
        assert_eq!(selected[0].source_rel, PathBuf::from("b.cpp"));
    }
}
