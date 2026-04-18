use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) use gix::protocol::handshake::Ref;
use tempfile::tempdir;

pub(super) use crate::spec::RevisionSelector;

pub(super) use super::checkout::checkout_detached_to_revision;
pub(super) use super::clone::{clone_bare_with_retry, clone_checkout_from_mirror};
pub(super) use super::fetch::fetch_origin_with_retry;
pub(super) use super::open::{open_bare_with_retry, open_checkout_with_retry};
pub(super) use super::probe::{
    default_remote_head_revision, probe_remote_target_revision_with_retry, remote_probe_options,
    remote_ref_target_revision,
};
pub(super) use super::retry::{is_retryable_remote_error_message, retry_delay_for_attempt};
pub(super) use super::types::RepositoryHandle;

mod mirror_materialization;
mod remote_probe;
mod repository_handle;
mod retry_policy;

const TEST_AUTHOR_NAME: &str = "backend-test";
const TEST_AUTHOR_EMAIL: &str = "backend-test@example.com";

pub(super) fn must<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

pub(super) fn object_id(hex: &[u8]) -> gix::hash::ObjectId {
    must(gix::hash::ObjectId::from_hex(hex), "parse object id")
}

pub(super) fn temp_dir() -> tempfile::TempDir {
    must(tempdir(), "create tempdir")
}

pub(super) fn init_test_repository(path: &Path) {
    must(
        Command::new("git").arg("init").arg(path).status(),
        "initialize repository",
    );
    configure_identity(path);
    must(
        fs::write(path.join("README.md"), "# fixture\n"),
        "write initial file",
    );
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg(".")
            .status(),
        "stage initial commit",
    );
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .status(),
        "create initial commit",
    );
}

pub(super) fn configure_identity(path: &Path) {
    for (key, value) in [
        ("user.name", TEST_AUTHOR_NAME),
        ("user.email", TEST_AUTHOR_EMAIL),
    ] {
        must(
            Command::new("git")
                .arg("-C")
                .arg(path)
                .arg("config")
                .arg(key)
                .arg(value)
                .status(),
            "configure repository identity",
        );
    }
}

pub(super) fn create_branch_and_commit(
    path: &Path,
    branch: &str,
    file: &str,
    content: &str,
    message: &str,
) {
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("checkout")
            .arg("-B")
            .arg(branch)
            .status(),
        "create branch",
    );
    let file_path = path.join(file);
    if let Some(parent) = file_path.parent() {
        must(fs::create_dir_all(parent), "create branch file parent");
    }
    must(fs::write(&file_path, content), "write branch file");
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg(file)
            .status(),
        "stage branch file",
    );
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .status(),
        "commit branch change",
    );
}

pub(super) fn create_annotated_tag(path: &Path, tag: &str, message: &str) {
    must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("tag")
            .arg("-a")
            .arg(tag)
            .arg("-m")
            .arg(message)
            .status(),
        "create annotated tag",
    );
}

pub(super) fn rev_parse(path: &Path, rev: &str) -> String {
    let output = must(
        Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("rev-parse")
            .arg(rev)
            .output(),
        "rev-parse revision",
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub(super) fn head_revision(path: &Path) -> String {
    rev_parse(path, "HEAD")
}

pub(super) fn describe_remote_refs(repository: &RepositoryHandle) -> String {
    format!("git_dir={}", repository.git_dir().display())
}
