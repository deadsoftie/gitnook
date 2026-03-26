use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use chrono::Utc;

use crate::config::{self, GitletConfig, GitletEntry};
use crate::exclude;

pub fn init(git_root: &Path, name: &str) -> anyhow::Result<()> {
    let gitlet_dir = git_root.join(".gitlet").join(name);

    if gitlet_dir.exists() {
        return Err(anyhow!(
            "gitlet '{}' already exists. Run 'gitlet list' to see all gitlets.",
            name
        ));
    }

    // Create the bare git repo for this gitlet (.gitlet/ is created as a side-effect)
    std::fs::create_dir_all(&gitlet_dir)
        .with_context(|| format!("failed to create {}", gitlet_dir.display()))?;
    git2::Repository::init_bare(&gitlet_dir)
        .with_context(|| format!("failed to init bare repo at {}", gitlet_dir.display()))?;

    // Create or update .gitlet/config.toml
    let gitlet_root = git_root.join(".gitlet");

    let mut cfg = if gitlet_root.join("config.toml").exists() {
        config::load(git_root)?
    } else {
        GitletConfig::default()
    };

    cfg.gitlets.insert(
        name.to_string(),
        GitletEntry {
            created: Utc::now().to_rfc3339(),
        },
    );

    if cfg.active.is_empty() {
        cfg.active = name.to_string();
    }

    config::save(git_root, &cfg)?;

    // Add .gitlet/ to .git/info/exclude (idempotent)
    exclude::add_exclusion(git_root, ".gitlet/")?;

    println!("Initialized gitlet '{}'", name);
    Ok(())
}

pub fn add(git_root: &Path, files: &[String], to: Option<&str>) -> anyhow::Result<()> {
    // Canonicalize git_root so strip_prefix works correctly on macOS where
    // current_dir() may return a symlinked path (e.g. /var → /private/var).
    let git_root = git_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", git_root.display()))?;
    let git_root = git_root.as_path();

    let cfg = config::load(git_root)?;
    let target = to.unwrap_or(&cfg.active).to_string();

    let gitlet_dir = git_root.join(".gitlet").join(&target);
    if !gitlet_dir.exists() {
        return Err(anyhow!("gitlet '{}' does not exist.", target));
    }

    let repo = git2::Repository::open(&gitlet_dir)
        .with_context(|| format!("failed to open gitlet repo at {}", gitlet_dir.display()))?;

    for file in files {
        // resolve_file canonicalizes; with git_root also canonical, strip_prefix is safe.
        let abs = resolve_file(file)?;
        let rel = abs
            .strip_prefix(git_root)
            .with_context(|| format!("'{}' is outside the git repo", file))?
            .to_path_buf();

        // Warn if tracked by the outer git
        if is_tracked_by_outer_git(git_root, &rel)? {
            eprintln!(
                "Warning: {} is tracked by git. To fully remove it run: git rm --cached {}",
                rel.display(),
                rel.display()
            );
        }

        // Error only if the file is owned by a *different* gitlet.
        // Re-adding to the same gitlet is how the user stages modifications.
        if let Some(owner) = find_owning_gitlet(git_root, &cfg, &rel)? {
            if owner != target {
                return Err(anyhow!(
                    "{} is already tracked by gitlet '{}'",
                    rel.display(),
                    owner
                ));
            }
        }

        // Stage in the target gitlet index.
        // Bare repos have no workdir, so we create a blob from the real file
        // and add it to the index manually. Use the canonical abs path directly.
        let blob_id = repo
            .blob_path(&abs)
            .with_context(|| format!("failed to create blob for {}", abs.display()))?;

        let mut index = repo.index().context("failed to get gitlet index")?;
        let entry = git2::IndexEntry {
            ctime: git2::IndexTime::new(0, 0),
            mtime: git2::IndexTime::new(0, 0),
            dev: 0,
            ino: 0,
            mode: 0o100644,
            uid: 0,
            gid: 0,
            file_size: 0,
            id: blob_id,
            flags: 0,
            flags_extended: 0,
            path: rel.to_string_lossy().into_owned().into_bytes(),
        };
        index.add(&entry).with_context(|| {
            format!("failed to stage {} in gitlet '{}'", rel.display(), target)
        })?;
        index.write().context("failed to write gitlet index")?;

        // Add to .git/info/exclude
        exclude::add_exclusion(git_root, &rel.to_string_lossy())?;

        println!("Added {} to gitlet '{}'", rel.display(), target);
    }

    Ok(())
}

pub fn remove(git_root: &Path, file: &str, to: Option<&str>) -> anyhow::Result<()> {
    // Canonicalize for consistency with `add` and correct strip_prefix on macOS.
    let git_root = git_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", git_root.display()))?;
    let git_root = git_root.as_path();

    let cfg = config::load(git_root)?;
    let target = to.unwrap_or(&cfg.active).to_string();

    let gitlet_dir = git_root.join(".gitlet").join(&target);
    if !gitlet_dir.exists() {
        return Err(anyhow!("gitlet '{}' does not exist.", target));
    }

    // Resolve path — file may have been deleted from disk, so don't require it to exist
    let rel = rel_path(git_root, file)?;

    // Verify the file is tracked by this gitlet
    let repo = git2::Repository::open(&gitlet_dir)
        .with_context(|| format!("failed to open gitlet repo at {}", gitlet_dir.display()))?;
    let mut index = repo.index().context("failed to get gitlet index")?;

    if index.get_path(&rel, 0).is_none() {
        return Err(anyhow!(
            "'{}' is not tracked by gitlet '{}'",
            rel.display(),
            target
        ));
    }

    index
        .remove_path(&rel)
        .with_context(|| format!("failed to remove {} from gitlet index", rel.display()))?;
    index.write().context("failed to write gitlet index")?;

    exclude::remove_exclusion(git_root, &rel.to_string_lossy())?;

    println!(
        "Removed {} from gitlet '{}'. The file is now visible to git.",
        rel.display(),
        target
    );
    Ok(())
}

pub fn commit(git_root: &Path, message: &str, to: Option<&str>) -> anyhow::Result<()> {
    let git_root = git_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", git_root.display()))?;
    let git_root = git_root.as_path();

    let cfg = config::load(git_root)?;
    let target = to.unwrap_or(&cfg.active).to_string();

    let gitlet_dir = git_root.join(".gitlet").join(&target);
    if !gitlet_dir.exists() {
        return Err(anyhow!("gitlet '{}' does not exist.", target));
    }

    let repo = git2::Repository::open(&gitlet_dir)
        .with_context(|| format!("failed to open gitlet repo at {}", gitlet_dir.display()))?;

    // Build the tree from the current index
    let mut index = repo.index().context("failed to get gitlet index")?;
    let tree_id = index.write_tree().context("failed to write index tree")?;
    let tree = repo.find_tree(tree_id).context("failed to find tree")?;

    // Read author/committer from the outer git config, with fallbacks
    let (author_name, author_email) = read_git_identity(git_root);
    let sig = git2::Signature::now(&author_name, &author_email)
        .context("failed to create git signature")?;

    // Create root commit or chained commit depending on whether HEAD resolves
    let oid = match repo.head() {
        Ok(head) => {
            let parent = head.peel_to_commit().context("failed to peel HEAD to commit")?;
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
                .context("failed to create commit")?
        }
        Err(_) => {
            // HEAD doesn't exist yet — this is the first commit
            repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                .context("failed to create root commit")?
        }
    };

    let short_sha = &oid.to_string()[..7];
    println!("[{}] {} {}", target, short_sha, message);
    Ok(())
}

/// Read user.name and user.email from the outer git config, falling back to defaults.
fn read_git_identity(git_root: &Path) -> (String, String) {
    let name;
    let email;

    match git2::Repository::discover(git_root).and_then(|r| r.config()) {
        Ok(cfg) => {
            name = cfg
                .get_string("user.name")
                .unwrap_or_else(|_| "gitlet user".to_string());
            email = cfg
                .get_string("user.email")
                .unwrap_or_else(|_| "gitlet@local".to_string());
        }
        Err(_) => {
            name = "gitlet user".to_string();
            email = "gitlet@local".to_string();
        }
    }

    (name, email)
}

/// Build a repo-relative path from a raw file argument without requiring the file to exist.
fn rel_path(git_root: &Path, file: &str) -> anyhow::Result<PathBuf> {
    let p = PathBuf::from(file);
    let abs = if p.is_absolute() {
        p
    } else {
        std::env::current_dir()?.join(&p)
    };
    // Normalise without hitting the filesystem (file may be deleted)
    let abs = normalize_path(&abs);
    abs.strip_prefix(git_root)
        .with_context(|| format!("'{}' is outside the git repo", file))
        .map(|p| p.to_path_buf())
}

/// Lexically normalise a path (resolve `.` and `..`) without hitting the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// Resolve a file argument to a canonical absolute path, erroring if it does not exist.
fn resolve_file(file: &str) -> anyhow::Result<PathBuf> {
    let p = PathBuf::from(file);
    let abs = if p.is_absolute() {
        p
    } else {
        std::env::current_dir()?.join(p)
    };
    // canonicalize resolves symlinks and `..` so strip_prefix against a
    // canonicalized git_root is always safe.
    abs.canonicalize()
        .with_context(|| format!("'{}' does not exist", file))
}

/// Check whether a relative path is currently staged in the outer git index.
fn is_tracked_by_outer_git(git_root: &Path, rel: &Path) -> anyhow::Result<bool> {
    let outer = git2::Repository::discover(git_root)
        .context("failed to open outer git repo")?;
    let index = outer.index().context("failed to read outer git index")?;
    Ok(index.get_path(rel, 0).is_some())
}

/// Return the name of the gitlet that already tracks `rel`, if any.
fn find_owning_gitlet(
    git_root: &Path,
    cfg: &GitletConfig,
    rel: &Path,
) -> anyhow::Result<Option<String>> {
    for name in cfg.gitlets.keys() {
        let gitlet_dir = git_root.join(".gitlet").join(name);
        if !gitlet_dir.exists() {
            continue;
        }
        let repo = git2::Repository::open(&gitlet_dir)
            .with_context(|| format!("failed to open gitlet repo '{}'", name))?;
        let index = repo
            .index()
            .with_context(|| format!("failed to read index for gitlet '{}'", name))?;
        if index.get_path(rel, 0).is_some() {
            return Ok(Some(name.clone()));
        }
    }
    Ok(None)
}
