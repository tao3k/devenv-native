use std::fs;
use std::path::Path;

use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};
use crate::test_support::{commit_all, init_git_repository};
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState, RepoLifecycleState, RepoSourceKind,
};

use crate::analyzers::cache::build_repository_analysis_cache_key;

#[test]
fn analysis_identity_uses_git_tracked_scope_for_git_checkouts() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_file(tempdir.path(), "src/lib.rs", "pub fn stable() {}\n");
    init_git_repository(tempdir.path());
    commit_all(tempdir.path(), "track source scope");

    for directory in [".cache", ".devenv", ".run", "node_modules", "target"] {
        write_file(
            tempdir.path(),
            &format!("{directory}/generated.rs"),
            "pub fn generated() -> i32 { 1 }\n",
        );
    }
    write_file(tempdir.path(), "notes/untracked.md", "outside git scope\n");

    let repository = repository_for(tempdir.path(), "repo-analysis-identity-ignored-dirs");
    let source = source_for(tempdir.path());
    let first_key =
        build_repository_analysis_cache_key(&repository, &source, Some(&metadata("rev-1")));

    for directory in [".cache", ".devenv", ".run", "node_modules", "target"] {
        write_file(
            tempdir.path(),
            &format!("{directory}/generated.rs"),
            "pub fn generated() -> i32 { 2 }\n",
        );
        write_file(
            tempdir.path(),
            &format!("{directory}/nested/new.md"),
            "generated cache surface\n",
        );
    }
    write_file(
        tempdir.path(),
        "notes/untracked.md",
        "still outside git scope\n",
    );
    let second_key =
        build_repository_analysis_cache_key(&repository, &source, Some(&metadata("rev-2")));

    assert_eq!(first_key.analysis_identity, second_key.analysis_identity);
}

#[test]
fn analysis_identity_keeps_data_directory_in_real_scenario_surface() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_file(tempdir.path(), "src/lib.rs", "pub fn stable() {}\n");
    write_file(
        tempdir.path(),
        ".data/WendaoGraph.jl/notebooks/search_strategy.jl",
        "query_score(x) = x\n",
    );
    init_git_repository(tempdir.path());
    commit_all(tempdir.path(), "track real scenario data surface");

    let repository = repository_for(tempdir.path(), "repo-analysis-identity-data-surface");
    let source = source_for(tempdir.path());
    let first_key =
        build_repository_analysis_cache_key(&repository, &source, Some(&metadata("rev-1")));

    write_file(
        tempdir.path(),
        ".data/WendaoGraph.jl/notebooks/search_strategy.jl",
        "query_score(x) = x + 1\n",
    );
    let second_key =
        build_repository_analysis_cache_key(&repository, &source, Some(&metadata("rev-2")));

    assert_ne!(first_key.analysis_identity, second_key.analysis_identity);
}

fn repository_for(root: &Path, id: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: id.to_string(),
        path: Some(root.to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: Vec::new(),
    }
}

fn source_for(root: &Path) -> MaterializedRepo {
    MaterializedRepo {
        checkout_root: root.to_path_buf(),
        mirror_root: None,
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        last_fetched_at: None,
        drift_state: RepoDriftState::NotApplicable,
        mirror_state: RepoLifecycleState::NotApplicable,
        checkout_state: RepoLifecycleState::Validated,
        source_kind: RepoSourceKind::LocalCheckout,
    }
}

fn metadata(revision: &str) -> LocalCheckoutMetadata {
    LocalCheckoutMetadata {
        revision: Some(revision.to_string()),
        remote_url: None,
    }
}

fn write_file(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
    }
    fs::write(&path, contents).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}
