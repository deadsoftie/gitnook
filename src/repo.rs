use std::path::{Path, PathBuf};

use anyhow::anyhow;

/// Walk up from `start` looking for a directory that contains `.git/`.
pub fn find_git_root_from(start: &Path) -> anyhow::Result<PathBuf> {
    let mut dir = start.to_path_buf();

    loop {
        if dir.join(".git").is_dir() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => return Err(anyhow!("Not inside a git repository")),
        }
    }
}

/// Find the git root starting from the process working directory.
pub fn find_git_root() -> anyhow::Result<PathBuf> {
    find_git_root_from(&std::env::current_dir()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_root_from_nested_subdir() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir(root.join(".git")).unwrap();

        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let found = find_git_root_from(&nested).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn errors_outside_git_repo() {
        let tmp = TempDir::new().unwrap();
        let err = find_git_root_from(tmp.path()).unwrap_err();
        assert_eq!(err.to_string(), "Not inside a git repository");
    }
}
