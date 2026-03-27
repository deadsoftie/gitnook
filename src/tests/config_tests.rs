use super::*;
use std::collections::HashMap;
use tempfile::TempDir;

fn make_config(active: &str, names: &[&str]) -> GitletConfig {
    let mut cfg = GitletConfig {
        active: active.to_string(),
        gitlets: HashMap::new(),
    };
    for &name in names {
        cfg.gitlets.insert(
            name.to_string(),
            GitletEntry {
                created: "2025-01-01T00:00:00Z".to_string(),
            },
        );
    }
    cfg
}

#[test]
fn missing_file_returns_descriptive_error() {
    let tmp = TempDir::new().unwrap();
    let err = load(tmp.path()).unwrap_err();
    assert!(
        err.to_string().contains("gitlet init"),
        "expected hint to run 'gitlet init', got: {err}"
    );
}

#[test]
fn save_creates_file_and_load_reads_it() {
    let tmp = TempDir::new().unwrap();
    let cfg = make_config("default", &["default"]);
    save(tmp.path(), &cfg).unwrap();

    let path = tmp.path().join(".gitlet").join("config.toml");
    assert!(path.exists(), "config.toml was not created");
}

#[test]
fn round_trip_preserves_fields() {
    let tmp = TempDir::new().unwrap();
    let cfg = make_config("secrets", &["secrets", "personal"]);
    save(tmp.path(), &cfg).unwrap();

    let loaded = load(tmp.path()).unwrap();
    assert_eq!(loaded.active, "secrets");
    assert!(loaded.gitlets.contains_key("secrets"));
    assert!(loaded.gitlets.contains_key("personal"));
    assert_eq!(loaded.gitlets["secrets"].created, "2025-01-01T00:00:00Z");
}

#[test]
fn get_active_returns_active_field() {
    let tmp = TempDir::new().unwrap();
    save(tmp.path(), &make_config("personal", &["personal"])).unwrap();
    assert_eq!(get_active(tmp.path()).unwrap(), "personal");
}

#[test]
fn set_active_updates_active_field() {
    let tmp = TempDir::new().unwrap();
    save(tmp.path(), &make_config("default", &["default", "work"])).unwrap();
    set_active(tmp.path(), "work").unwrap();
    assert_eq!(get_active(tmp.path()).unwrap(), "work");
}

#[test]
fn save_is_atomic_temp_file_removed() {
    let tmp = TempDir::new().unwrap();
    let cfg = make_config("default", &["default"]);
    save(tmp.path(), &cfg).unwrap();

    let tmp_path = tmp.path().join(".gitlet").join("config.toml.tmp");
    assert!(!tmp_path.exists(), "temp file should be removed after save");
}
