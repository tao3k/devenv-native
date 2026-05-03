use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::analyzers::PluginRegistry;
use crate::analyzers::service::analysis::{
    analyze_registered_repository_bundle_with_registry,
    analyze_registered_repository_target_file_with_registry,
};
use crate::analyzers::{RegisteredRepository, RepositoryRefreshPolicy};
use crate::analyzers::{RepositoryPluginConfig, resolve_registered_repository_source};
use crate::test_support::{commit_all, init_git_repository};
use xiuxian_git_repo::SyncMode;

use super::support::{CachedTargetFilePlugin, CountingJuliaPlugin};

#[test]
fn analyze_target_file_reuses_existing_managed_checkout_without_remote_probe() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_dir = tempdir.path().join("fixture-source");
    fs::create_dir_all(source_dir.join("src"))
        .unwrap_or_else(|error| panic!("create source src: {error}"));
    init_git_repository(&source_dir);
    fs::write(
        source_dir.join("Project.toml"),
        "name = \"FixturePkg\"\nversion = \"0.1.0\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        source_dir.join("src/FixturePkg.jl"),
        "module FixturePkg\nsolve(x) = x\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    commit_all(&source_dir, "initial");

    let remote_dir = tempdir.path().join("fixture-remote.git");
    let clone_status = Command::new("git")
        .args([
            "clone",
            "--bare",
            source_dir
                .to_str()
                .unwrap_or_else(|| panic!("source path utf8")),
            remote_dir
                .to_str()
                .unwrap_or_else(|| panic!("remote path utf8")),
        ])
        .status()
        .unwrap_or_else(|error| panic!("clone bare remote: {error}"));
    assert!(clone_status.success(), "clone bare remote should succeed");

    let repository = RegisteredRepository {
        id: format!("managed-target-file-{}", std::process::id()),
        path: None,
        url: Some(remote_dir.display().to_string()),
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };

    let materialized =
        resolve_registered_repository_source(&repository, tempdir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("materialize managed checkout: {error}"));
    assert!(materialized.checkout_root.is_dir());

    fs::remove_dir_all(&remote_dir)
        .unwrap_or_else(|error| panic!("remove bare remote to block ensure path: {error}"));

    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CountingJuliaPlugin {
            calls: Arc::clone(&calls),
        })
        .unwrap_or_else(|error| panic!("register test plugin: {error}"));

    let analysis = analyze_registered_repository_target_file_with_registry(
        &repository,
        tempdir.path(),
        &registry,
        "src/FixturePkg.jl",
    )
    .unwrap_or_else(|error| panic!("target-file analysis should reuse checkout: {error}"));

    assert_eq!(analysis.modules.len(), 1);
    assert_eq!(analysis.modules[0].path, "src/FixturePkg.jl");
    assert_eq!(analysis.symbols.len(), 1);
    assert_eq!(analysis.symbols[0].path, "src/FixturePkg.jl");
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let _ = fs::remove_dir_all(&materialized.checkout_root);
    if let Some(mirror_root) = materialized.mirror_root.as_ref() {
        let _ = fs::remove_dir_all(mirror_root);
    }
}

#[test]
fn analyze_target_file_reuses_ready_cached_analysis_before_file_probe() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("Project.toml"),
        "name = \"FixturePkg\"\nversion = \"0.1.0\"\n",
    )
    .unwrap_or_else(|error| panic!("write Project.toml: {error}"));
    fs::write(
        tempdir.path().join("src/FixturePkg.jl"),
        "module FixturePkg\nusing LinearAlgebra\nsolve(x) = x\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    commit_all(tempdir.path(), "initial");

    let repository = RegisteredRepository {
        id: "cached-target-file".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        git_ref: None,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    let repository_calls = Arc::new(AtomicUsize::new(0));
    let file_calls = Arc::new(AtomicUsize::new(0));
    let mut registry = PluginRegistry::new();
    registry
        .register(CachedTargetFilePlugin {
            repository_calls: Arc::clone(&repository_calls),
            file_calls: Arc::clone(&file_calls),
        })
        .unwrap_or_else(|error| panic!("register cached target-file plugin: {error}"));

    analyze_registered_repository_bundle_with_registry(&repository, tempdir.path(), &registry)
        .unwrap_or_else(|error| panic!("seed cached analysis: {error}"));

    let analysis = analyze_registered_repository_target_file_with_registry(
        &repository,
        tempdir.path(),
        &registry,
        "src/FixturePkg.jl",
    )
    .unwrap_or_else(|error| panic!("target-file analysis should reuse cache: {error}"));

    assert_eq!(repository_calls.load(Ordering::SeqCst), 1);
    assert_eq!(file_calls.load(Ordering::SeqCst), 0);
    assert_eq!(analysis.modules.len(), 1);
    assert_eq!(analysis.modules[0].path, "src/FixturePkg.jl");
    assert_eq!(analysis.symbols.len(), 1);
    assert_eq!(analysis.symbols[0].path, "src/FixturePkg.jl");
    assert_eq!(analysis.imports.len(), 1);
    assert_eq!(analysis.imports[0].import_name, "LinearAlgebra");
}
