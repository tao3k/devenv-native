use std::fs;

use serde_json::json;

use super::{
    RepositoryPluginConfig, RepositoryRef, RepositoryRefreshPolicy, load_repo_intelligence_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn load_repo_intelligence_config_parses_inline_plugin_config() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = temp.path().join("repos").join("sample");
    fs::create_dir_all(&repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.sample]
root = "repos/sample"
refresh = "manual"
plugins = [
  "repo-content",
  { id = "julia-code-parser", flight_transport = { base_url = "http://127.0.0.1:8815", route = "/rerank", timeout_secs = 15 } }
]
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&config_path), temp.path())?;
    assert_eq!(config.repos.len(), 1);
    let repository = &config.repos[0];
    assert_eq!(repository.id, "sample");
    assert_eq!(repository.refresh, RepositoryRefreshPolicy::Manual);
    assert_eq!(repository.path.as_deref(), Some(repo_dir.as_path()));
    assert_eq!(
        repository.plugins,
        vec![
            RepositoryPluginConfig::Config {
                id: "julia-code-parser".to_string(),
                options: json!({
                    "flight_transport": {
                        "base_url": "http://127.0.0.1:8815",
                        "route": "/rerank",
                        "timeout_secs": 15,
                    }
                }),
            },
            RepositoryPluginConfig::Id("markdown-parser".to_string()),
        ]
    );
    Ok(())
}

#[test]
fn load_repo_intelligence_config_rejects_empty_inline_plugin_id() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = temp.path().join("repos").join("sample");
    fs::create_dir_all(&repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.sample]
root = "repos/sample"
plugins = [{ id = "   ", flight_transport = { base_url = "http://127.0.0.1:8815" } }]
"#,
    )?;

    let Err(error) = load_repo_intelligence_config(Some(&config_path), temp.path()) else {
        panic!("expected empty plugin id to be rejected");
    };
    assert_eq!(
        error.to_string(),
        format!(
            "repo intelligence config load failed: failed to parse `{}`: repo `sample` plugin id cannot be empty",
            config_path.display()
        )
    );
    Ok(())
}

#[test]
fn load_repo_intelligence_config_parses_prefixed_repository_refs() -> TestResult {
    let temp = tempfile::tempdir()?;
    let commit_repo_dir = temp.path().join("repos").join("commit-sample");
    let tag_repo_dir = temp.path().join("repos").join("tag-sample");
    fs::create_dir_all(&commit_repo_dir)?;
    fs::create_dir_all(&tag_repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.commit-sample]
root = "repos/commit-sample"
ref = "commit:abc123"
plugins = ["julia-code-parser"]

[sources.projects.tag-sample]
root = "repos/tag-sample"
ref = "tag:v1.2.3"
plugins = ["julia-code-parser"]
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&config_path), temp.path())?;
    assert_eq!(config.repos.len(), 2);
    assert_eq!(
        config.repos[0].git_ref,
        Some(RepositoryRef::Commit("abc123".to_string()))
    );
    assert_eq!(
        config.repos[1].git_ref,
        Some(RepositoryRef::Tag("v1.2.3".to_string()))
    );
    Ok(())
}

#[test]
fn load_repo_intelligence_config_defaults_markdown_parser_for_source_repositories() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = temp.path().join("repos").join("knowledge");
    fs::create_dir_all(&repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.knowledge]
root = "repos/knowledge"
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&config_path), temp.path())?;

    assert_eq!(config.repos.len(), 1);
    assert_eq!(config.repos[0].id, "knowledge");
    assert_eq!(config.repos[0].path.as_deref(), Some(repo_dir.as_path()));
    assert_eq!(
        config.repos[0].plugins,
        vec![RepositoryPluginConfig::Id("markdown-parser".to_string())]
    );
    Ok(())
}

#[test]
fn load_repo_intelligence_config_reads_overlay_importing_base() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = temp.path().join("repos").join("sample");
    fs::create_dir_all(&repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    let overlay_path = temp.path().join("wendao.studio.overlay.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.sample]
root = "repos/sample"
plugins = ["julia-code-parser"]
"#,
    )?;
    fs::write(
        &overlay_path,
        r#"imports = ["wendao.toml"]

[sources.projects.sample]
refresh = "manual"
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&overlay_path), temp.path())?;
    assert_eq!(config.repos.len(), 1);
    assert_eq!(config.repos[0].refresh, RepositoryRefreshPolicy::Manual);
    assert_eq!(config.repos[0].path.as_deref(), Some(repo_dir.as_path()));
    Ok(())
}

#[test]
fn load_repo_intelligence_config_filters_search_only_plugins_and_adds_markdown_parser() -> TestResult
{
    let temp = tempfile::tempdir()?;
    let mixed_repo_dir = temp.path().join("repos").join("mixed");
    let search_only_repo_dir = temp.path().join("repos").join("search-only");
    fs::create_dir_all(&mixed_repo_dir)?;
    fs::create_dir_all(&search_only_repo_dir)?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        r#"[sources.projects.mixed]
root = "repos/mixed"
plugins = ["repo-content", "julia-code-parser"]

[sources.projects.search-only]
root = "repos/search-only"
plugins = ["repo-content"]
"#,
    )?;

    let config = load_repo_intelligence_config(Some(&config_path), temp.path())?;

    assert_eq!(config.repos.len(), 2);
    let mixed = config
        .repos
        .iter()
        .find(|repository| repository.id == "mixed")
        .unwrap_or_else(|| panic!("mixed repo should be loaded"));
    assert_eq!(
        mixed.plugins,
        vec![
            RepositoryPluginConfig::Id("julia-code-parser".to_string()),
            RepositoryPluginConfig::Id("markdown-parser".to_string()),
        ]
    );
    let search_only = config
        .repos
        .iter()
        .find(|repository| repository.id == "search-only")
        .unwrap_or_else(|| panic!("search-only repo should receive the markdown parser default"));
    assert_eq!(
        search_only.plugins,
        vec![RepositoryPluginConfig::Id("markdown-parser".to_string())]
    );
    Ok(())
}
