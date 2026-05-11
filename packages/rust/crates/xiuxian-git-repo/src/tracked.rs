//! Git-tracked file scope helpers.

use std::path::Path;

use gix::bstr::ByteSlice;
use gix::index::entry::{Mode, Stage};

use crate::error::{RepoError, RepoErrorKind};

/// Lists Git-tracked file paths for a checkout, relative to the repository root.
///
/// # Errors
///
/// Returns an error when the checkout cannot be opened as a Git repository or
/// when its index cannot be read.
pub fn list_tracked_file_paths(checkout_root: &Path) -> Result<Vec<String>, RepoError> {
    let repository = gix::open(checkout_root).map_err(|error| {
        repo_error_from_message(
            format!("open git checkout `{}`: {error}", checkout_root.display()),
            RepoErrorKind::InvalidPath,
        )
    })?;
    if repository.is_bare() {
        return Err(RepoError::new(
            RepoErrorKind::Unsupported,
            format!("git checkout `{}` is bare", checkout_root.display()),
        ));
    }

    let index = repository.open_index().map_err(|error| {
        repo_error_from_message(
            format!("read git index for `{}`: {error}", checkout_root.display()),
            RepoErrorKind::RepositoryCorrupt,
        )
    })?;
    let mut paths = index
        .entries_with_paths_by_filter_map(|path, entry| {
            is_file_scope_entry(entry).then(|| path.to_str().ok().map(str::to_owned))?
        })
        .map(|(_path, relative_path)| relative_path)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn is_file_scope_entry(entry: &gix::index::Entry) -> bool {
    entry.stage() == Stage::Unconflicted
        && matches!(
            entry.mode,
            mode if mode == Mode::FILE || mode == Mode::FILE_EXECUTABLE || mode == Mode::SYMLINK
        )
}

fn repo_error_from_message(message: String, fallback: RepoErrorKind) -> RepoError {
    let kind = match RepoError::classify_message(message.as_str()) {
        RepoErrorKind::Permanent => fallback,
        kind => kind,
    };
    RepoError::new(kind, message)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::list_tracked_file_paths;

    #[test]
    fn list_tracked_file_paths_returns_git_index_scope() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
        init_git(tempdir.path());
        write_file(tempdir.path(), "src/lib.rs", "pub fn stable() {}\n");
        write_file(tempdir.path(), ".data/notebook.jl", "score(x) = x\n");
        git(tempdir.path(), &["add", "src/lib.rs", ".data/notebook.jl"]);
        git(tempdir.path(), &["commit", "-m", "track scope"]);
        write_file(
            tempdir.path(),
            "target/generated.rs",
            "pub fn generated() {}\n",
        );
        write_file(tempdir.path(), "notes/untracked.md", "outside scope\n");

        let paths = list_tracked_file_paths(tempdir.path())
            .unwrap_or_else(|error| panic!("list tracked paths: {error}"));

        assert_eq!(
            paths,
            vec![".data/notebook.jl".to_string(), "src/lib.rs".to_string()]
        );
    }

    #[test]
    fn list_tracked_file_paths_rejects_non_git_directory() {
        let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));

        let error = list_tracked_file_paths(tempdir.path())
            .expect_err("non-git directory should not expose tracked scope");

        assert!(!error.message.is_empty());
    }

    fn init_git(root: &Path) {
        git(root, &["init"]);
        git(root, &["config", "user.name", "tracked-test"]);
        git(root, &["config", "user.email", "tracked-test@example.com"]);
    }

    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("run git {}: {error}", args.join(" ")));
        if output.status.success() {
            return;
        }
        panic!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write_file(root: &Path, relative_path: &str, contents: &str) {
        let path = root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
        }
        fs::write(&path, contents)
            .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    }
}
