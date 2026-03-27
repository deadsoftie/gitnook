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
