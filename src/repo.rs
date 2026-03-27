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
#[path = "tests/repo_tests.rs"]
mod tests;
