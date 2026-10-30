//! Ignore-aware selection of explicit files and directory contents.

use crate::lang_config::LangConfig;
use ignore::WalkBuilder;
use std::collections::{BTreeSet, HashMap};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct SelectedFile {
    /// Display path, independent of the canonical identity used for deduplication.
    pub path: PathBuf,
    pub explicit: bool,
}

#[derive(Debug)]
pub struct InputSelection {
    pub files: Vec<SelectedFile>,
    /// Common ancestor used only for labels. None means different filesystem roots.
    pub naming_base: Option<PathBuf>,
}

impl InputSelection {
    pub fn collect(paths: &[PathBuf], no_ignore: bool) -> io::Result<Self> {
        let mut files: HashMap<PathBuf, SelectedFile> = HashMap::new();
        let mut bases = Vec::new();
        for path in paths {
            let path = std::path::absolute(path)?;
            let metadata = path.metadata().map_err(|e| path_error(&path, e))?;
            if metadata.is_file() {
                bases.push(path.parent().unwrap_or(&path).to_path_buf());
                insert_file(&mut files, path, true)?;
            } else if metadata.is_dir() {
                bases.push(path.clone());
                let mut builder = WalkBuilder::new(&path);
                builder.require_git(false).standard_filters(!no_ignore);
                builder.sort_by_file_path(|a, b| a.cmp(b));
                for entry in builder.build() {
                    let entry = entry.map_err(io::Error::other)?;
                    if entry.file_type().is_some_and(|kind| kind.is_file()) {
                        insert_file(&mut files, entry.into_path(), false)?;
                    }
                }
            } else {
                return Err(path_error(&path, "expected a file or directory"));
            }
        }
        let mut files: Vec<_> = files.into_values().collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self {
            files,
            naming_base: common_ancestor(&bases),
        })
    }

    /// Infer one language, rejecting mixed languages and ambiguous extensions.
    pub fn detect_language(&self) -> io::Result<String> {
        let configs = LangConfig::all_configs();
        let mut languages = BTreeSet::new();
        for file in &self.files {
            for (name, config) in &configs {
                if matches_language(&file.path, config) {
                    languages.insert(name.clone());
                }
            }
        }
        if languages.len() != 1 {
            return Err(io::Error::other(
                "Could not determine one language for the selection. Please specify with --lang",
            ));
        }
        Ok(languages.into_iter().next().unwrap())
    }
}

pub(crate) fn matches_language(path: &Path, config: &LangConfig) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            config.file_extensions.as_ref().is_some_and(|extensions| {
                extensions
                    .iter()
                    .any(|candidate| candidate.eq_ignore_ascii_case(ext))
            })
        })
}

fn insert_file(
    files: &mut HashMap<PathBuf, SelectedFile>,
    path: PathBuf,
    explicit: bool,
) -> io::Result<()> {
    let identity = path.canonicalize().map_err(|e| path_error(&path, e))?;
    files
        .entry(identity)
        .and_modify(|file| file.explicit |= explicit)
        .or_insert(SelectedFile { path, explicit });
    Ok(())
}

fn common_ancestor(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut base = paths.first()?.clone();
    for path in &paths[1..] {
        while !path.starts_with(&base) {
            if !base.pop() {
                return None;
            }
        }
    }
    Some(base)
}

fn path_error(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("{}: {error}", path.display()))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn different_drives_have_no_common_ancestor() {
        assert_eq!(
            common_ancestor(&[PathBuf::from(r"C:\models"), PathBuf::from(r"D:\service")]),
            None
        );
    }
}
