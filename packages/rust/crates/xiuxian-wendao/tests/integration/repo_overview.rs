//! Integration tests for Repo Intelligence overview flow.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::json;
use uuid::Uuid;
use xiuxian_io::PrjDirs;
use xiuxian_wendao::analyzers::{
    RepoIntelligenceError, RepoOverviewQuery, RepositoryRefreshPolicy,
    analyze_repository_from_config, bootstrap_builtin_registry, load_repo_intelligence_config,
    repo_overview_from_config,
};

use crate::support::repo_fixture::{git_is_bare_repository, git_remote_url};
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use xiuxian_wendao::analyzers::{
    ExampleSearchQuery, ModuleSearchQuery, example_search_from_config, module_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn config_parses_relative_repo_paths_against_config_dir() -> TestResult {
    let temp = tempfile::tempdir()?;
    let config_dir = temp.path().join("config");
    let repo_dir = temp.path().join("repos").join("sample");
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&repo_dir)?;

    let config_path = config_dir.join("wendao.toml");
    fs::write(
        &config_path,
        r#"[link_graph.projects.sample]
root = "../repos/sample"
plugins = ["julia"]
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&config_path), temp.path())?;
    assert_eq!(config.repos.len(), 1);
    assert_eq!(config.repos[0].id, "sample");
    assert_eq!(config.repos[0].path.as_deref(), Some(repo_dir.as_path()));
    assert_eq!(config.repos[0].refresh, RepositoryRefreshPolicy::Fetch);
    Ok(())
}

#[test]
fn builtin_registry_contains_julia_plugin() -> TestResult {
    let registry = bootstrap_builtin_registry()?;
    assert!(registry.plugin_ids().contains(&"julia"));
    Ok(())
}

#[test]
fn julia_analyzer_builds_repo_overview_from_local_repo() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "SamplePkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "sample")?;

    let analysis = analyze_repository_from_config("sample", Some(&config_path), temp.path())?;
    let overview = repo_overview_from_config(
        &RepoOverviewQuery {
            repo_id: "sample".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;

    let mut payload = json!({
        "analysis": {
            "repository": analysis.repository,
            "modules": analysis.modules,
            "symbols": analysis.symbols,
            "examples": analysis.examples,
            "docs": analysis.docs,
            "relations": analysis.relations,
            "diagnostics": analysis.diagnostics,
        },
        "overview": overview,
    });
    redact_repo_root(&mut payload);
    redact_repo_revision(&mut payload);
    assert_repo_json_snapshot("repo_overview_analysis", payload);
    Ok(())
}

#[test]
fn modelica_plugin_builds_repo_overview_and_search_results_from_local_repo() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "DemoLib")?;
    let config_path = temp.path().join("modelica-demo.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.modelica-demo]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let analysis =
        analyze_repository_from_config("modelica-demo", Some(&config_path), temp.path())?;
    let overview = repo_overview_from_config(
        &RepoOverviewQuery {
            repo_id: "modelica-demo".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let module_search = module_search_from_config(
        &ModuleSearchQuery {
            repo_id: "modelica-demo".to_string(),
            query: "Controllers".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;
    let example_search = example_search_from_config(
        &ExampleSearchQuery {
            repo_id: "modelica-demo".to_string(),
            query: "Step".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    let mut payload = json!({
        "analysis": {
            "repository": analysis.repository,
            "modules": analysis.modules,
            "symbols": analysis.symbols,
            "examples": analysis.examples,
            "docs": analysis.docs,
            "relations": analysis.relations,
            "diagnostics": analysis.diagnostics,
        },
        "overview": overview,
        "module_search": module_search,
        "example_search": example_search,
    });
    redact_repo_root(&mut payload);
    redact_repo_revision(&mut payload);
    assert_repo_json_snapshot("repo_overview_modelica_analysis", payload);
    Ok(())
}

#[test]
fn julia_analyzer_falls_back_when_expected_root_file_is_missing() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "FallbackPkg", false)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "fallback")?;

    let analysis = analyze_repository_from_config("fallback", Some(&config_path), temp.path())?;

    let mut payload = json!({
        "repository": analysis.repository,
        "modules": analysis.modules,
        "symbols": analysis.symbols,
        "examples": analysis.examples,
        "docs": analysis.docs,
        "relations": analysis.relations,
        "diagnostics": analysis.diagnostics,
    });
    redact_repo_root(&mut payload);
    redact_repo_revision(&mut payload);
    assert_repo_json_snapshot("repo_overview_fallback", payload);
    Ok(())
}

#[test]
fn cli_repo_overview_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-sample")?;

    let output = Command::new(env!("CARGO_BIN_EXE_wendao"))
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("overview")
        .arg("--repo")
        .arg("cli-sample")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let mut payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    redact_repo_revision(&mut payload);
    assert_repo_json_snapshot("repo_overview_cli_json", payload);
    Ok(())
}

#[test]
fn repo_analysis_clones_managed_checkout_from_url() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed")?;
    let config_path = write_repo_url_config(temp.path(), &source_repo, "managed")?;

    let analysis = analyze_repository_from_config("managed", Some(&config_path), temp.path())?;
    let mirror_root = managed_mirror_root(temp.path(), "managed");
    let managed_root = managed_checkout_root(temp.path(), "managed");
    let cache_listing = list_cache_tree(repo_cache_root(temp.path()).as_path());
    assert!(
        mirror_root.is_dir(),
        "missing mirror root: {} | cache entries: {:?}",
        mirror_root.display(),
        cache_listing
    );
    assert!(
        managed_root.is_dir(),
        "missing managed checkout root: {}",
        managed_root.display()
    );
    let canonical_mirror_root = fs::canonicalize(&mirror_root)?;
    assert!(git_is_bare_repository(&mirror_root)?);
    assert_eq!(
        git_remote_url(&managed_root, "origin")?,
        canonical_mirror_root.display().to_string()
    );

    let mut payload = json!({
        "repository": analysis.repository,
        "modules": analysis.modules,
        "symbols": analysis.symbols,
        "examples": analysis.examples,
        "docs": analysis.docs,
        "relations": analysis.relations,
        "diagnostics": analysis.diagnostics,
    });
    redact_repo_root(&mut payload);
    redact_repo_url(&mut payload);
    redact_repo_revision(&mut payload);
    assert_repo_json_snapshot("repo_overview_managed_clone", payload);
    Ok(())
}

#[test]
fn managed_checkout_fetches_branch_updates() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedFetchPkg", true)?;
    let repo_id = format!("managed-fetch-{}", Uuid::new_v4());
    clear_managed_repo_cache(temp.path(), repo_id.as_str())?;
    let config_path = write_repo_url_config_with_ref(
        temp.path(),
        &source_repo,
        repo_id.as_str(),
        Some("main"),
        None,
    )?;

    let first = analyze_repository_from_config(repo_id.as_str(), Some(&config_path), temp.path())?;
    append_repo_file_and_commit(
        &source_repo,
        "docs/advanced.md",
        "# Advanced\n",
        "add advanced guide",
    )?;
    let second = analyze_repository_from_config(repo_id.as_str(), Some(&config_path), temp.path())?;

    let Some(first_repository) = first.repository else {
        panic!("repository record");
    };
    let Some(second_repository) = second.repository else {
        panic!("repository record");
    };
    assert_ne!(first_repository.revision, second_repository.revision);
    assert_eq!(first.docs.len() + 1, second.docs.len());
    Ok(())
}

#[test]
fn managed_checkout_respects_manual_refresh_policy() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_repo = create_sample_julia_repo(temp.path(), "ManagedManualPkg", true)?;
    clear_managed_repo_cache(temp.path(), "managed-manual")?;
    let config_path = write_repo_url_config_with_ref(
        temp.path(),
        &source_repo,
        "managed-manual",
        Some("main"),
        Some("manual"),
    )?;

    let first = analyze_repository_from_config("managed-manual", Some(&config_path), temp.path())?;
    append_repo_file_and_commit(
        &source_repo,
        "docs/manual-only.md",
        "# Manual only\n",
        "add manual-only guide",
    )?;
    let second = analyze_repository_from_config("managed-manual", Some(&config_path), temp.path())?;

    let Some(first_repository) = first.repository else {
        panic!("repository record");
    };
    let Some(second_repository) = second.repository else {
        panic!("repository record");
    };
    assert_eq!(first_repository.revision, second_repository.revision);
    assert_eq!(first.docs.len(), second.docs.len());
    Ok(())
}

#[test]
fn repo_analysis_rejects_non_git_directories() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = temp.path().join("not-a-checkout");
    fs::create_dir_all(&repo_dir)?;

    let config_path = write_repo_config(temp.path(), &repo_dir, "not-a-checkout")?;
    let Err(error) =
        analyze_repository_from_config("not-a-checkout", Some(&config_path), temp.path())
    else {
        panic!("non-git directories should be rejected");
    };

    match error {
        RepoIntelligenceError::InvalidRepositoryPath {
            repo_id,
            path,
            reason,
        } => {
            assert_eq!(repo_id, "not-a-checkout");
            assert!(path.ends_with("not-a-checkout"), "{path}");
            assert!(reason.contains("git checkout"), "{reason}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

fn redact_repo_root(value: &mut serde_json::Value) {
    if let Some(path) = value.pointer_mut("/analysis/repository/path") {
        *path = serde_json::Value::String("[repo-root]".to_string());
        return;
    }

    if let Some(path) = value.pointer_mut("/repository/path") {
        *path = serde_json::Value::String("[repo-root]".to_string());
    }
}

fn redact_repo_url(value: &mut serde_json::Value) {
    if let Some(url) = value.pointer_mut("/analysis/repository/url") {
        *url = serde_json::Value::String("[repo-url]".to_string());
        return;
    }

    if let Some(url) = value.pointer_mut("/repository/url") {
        *url = serde_json::Value::String("[repo-url]".to_string());
    }
}

fn redact_repo_revision(value: &mut serde_json::Value) {
    for pointer in [
        "/analysis/repository/revision",
        "/overview/revision",
        "/repository/revision",
        "/revision",
    ] {
        if let Some(revision) = value.pointer_mut(pointer)
            && !revision.is_null()
        {
            *revision = serde_json::Value::String("[revision]".to_string());
        }
    }
}

fn write_repo_url_config(
    base: &Path,
    repo_url: &Path,
    repo_id: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    write_repo_url_config_with_ref(base, repo_url, repo_id, None, None)
}

fn write_repo_url_config_with_ref(
    base: &Path,
    repo_url: &Path,
    repo_id: &str,
    git_ref: Option<&str>,
    refresh: Option<&str>,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let config_path = base.join(format!("{repo_id}.wendao.toml"));
    let ref_block = git_ref
        .map(|value| format!("ref = \"{value}\"\n"))
        .unwrap_or_default();
    let refresh_block = refresh
        .map(|value| format!("refresh = \"{value}\"\n"))
        .unwrap_or_default();
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.{repo_id}]
url = "{}"
{}{}plugins = ["julia"]
"#,
            repo_url.display(),
            ref_block,
            refresh_block
        ),
    )?;
    Ok(config_path)
}

fn append_repo_file_and_commit(
    repo_dir: &Path,
    relative_path: &str,
    contents: &str,
    message: &str,
) -> TestResult {
    crate::support::repo_fixture::append_repo_file_and_commit(
        repo_dir,
        relative_path,
        contents,
        message,
    )
}

fn repo_cache_root(_cwd: &Path) -> std::path::PathBuf {
    PrjDirs::data_home()
        .join("xiuxian-wendao")
        .join("repo-intelligence")
}

fn managed_mirror_root(cwd: &Path, repo_id: &str) -> std::path::PathBuf {
    repo_cache_root(cwd)
        .join("mirrors")
        .join(format!("{}.git", sanitize_repo_id(repo_id)))
}

fn managed_checkout_root(cwd: &Path, repo_id: &str) -> std::path::PathBuf {
    repo_cache_root(cwd)
        .join("repos")
        .join(sanitize_repo_id(repo_id))
}

fn sanitize_repo_id(repo_id: &str) -> String {
    repo_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn list_cache_tree(root: &Path) -> Vec<String> {
    if !root.exists() {
        return Vec::new();
    }

    walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().display().to_string())
        .collect()
}

fn clear_managed_repo_cache(cwd: &Path, repo_id: &str) -> TestResult {
    for root in [
        managed_mirror_root(cwd, repo_id),
        managed_checkout_root(cwd, repo_id),
    ] {
        if root.exists() {
            fs::remove_dir_all(root)?;
        }
    }
    Ok(())
}
