use std::fs;
use std::path::Path;
use std::process::Command;

use xiuxian_git_repo::list_tracked_file_paths;

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

    let error = match list_tracked_file_paths(tempdir.path()) {
        Ok(paths) => panic!("non-git directory returned tracked paths: {paths:?}"),
        Err(error) => error,
    };

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
    fs::write(&path, contents).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}
