use std::fs;

use crate::studio::router::{
    load_document_extract_endpoint_from_wendao_toml, load_episteme_registry_from_wendao_toml,
    load_model_routing_config_from_wendao_toml, load_ui_config_from_wendao_toml,
    load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml,
    studio_wendao_overlay_toml_path, studio_wendao_toml_path,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn load_ui_config_from_wendao_toml_accepts_inline_repo_plugin_config() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("wendao.toml"),
        r#"[sources.projects.sample]
root = "."
plugins = [
  "ast-grep",
  { id = "julia-code-parser", flight_transport = { base_url = "http://127.0.0.1:8815" } }
]
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load");
    };
    assert_eq!(config.repo_projects.len(), 1);
    assert_eq!(config.repo_projects[0].id, "sample");
    assert_eq!(
        config.repo_projects[0].plugins,
        vec![
            "ast-grep".to_string(),
            "julia-code-parser".to_string(),
            "markdown-parser".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn load_ui_config_from_wendao_toml_defaults_markdown_parser_for_repo_projects() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("wendao.toml"),
        r#"[sources.projects.knowledge]
root = "."
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load");
    };
    assert_eq!(config.repo_projects.len(), 1);
    assert_eq!(config.repo_projects[0].id, "knowledge");
    assert_eq!(
        config.repo_projects[0].plugins,
        vec!["markdown-parser".to_string()]
    );
    Ok(())
}

#[test]
fn load_ui_config_from_wendao_toml_maps_global_link_graph_include_dirs_to_main_project()
-> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("wendao.toml"),
        r#"[link_graph]
include_dirs = ["docs", "./semantic", "packages/rust/crates/xiuxian-wendao"]

[sources.projects.main]
root = "."
plugins = []
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load");
    };
    assert_eq!(config.projects.len(), 1);
    assert_eq!(config.projects[0].name, "main");
    assert_eq!(config.projects[0].root, ".");
    assert_eq!(
        config.projects[0].dirs,
        vec![
            "docs".to_string(),
            "semantic".to_string(),
            "packages/rust/crates/xiuxian-wendao".to_string(),
        ]
    );
    assert_eq!(config.repo_projects.len(), 1);
    assert_eq!(config.repo_projects[0].id, "main");
    Ok(())
}

#[test]
fn load_ui_config_from_wendao_toml_prefers_overlay_importing_base() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[sources.projects.kernel]
root = "."
dirs = ["docs"]
"#,
    )?;
    fs::write(
        studio_wendao_overlay_toml_path(temp.path()),
        r#"imports = ["wendao.toml"]

[sources.projects.kernel]
root = "."
dirs = ["docs", "src"]
"#,
    )?;

    let Some(config) = load_ui_config_from_wendao_toml(temp.path()) else {
        panic!("ui config should load from the persisted base config");
    };
    assert_eq!(config.projects.len(), 1);
    assert_eq!(config.projects[0].name, "kernel");
    assert_eq!(
        config.projects[0].dirs,
        vec!["docs".to_string(), "src".to_string()]
    );
    Ok(())
}

#[test]
fn load_document_extract_endpoint_from_wendao_toml_reads_effective_config() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[document_extract]
endpoint = "http://127.0.0.1:50051/"
"#,
    )?;

    assert_eq!(
        load_document_extract_endpoint_from_wendao_toml(temp.path()).as_deref(),
        Some("http://127.0.0.1:50051")
    );
    Ok(())
}

#[test]
fn load_model_routing_config_from_wendao_toml_returns_none_after_marlin_migration() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[model_routing]
mode = "deterministic"
default_provider = "openrouter"

[model_routing.chat]
model = "deepseek/deepseek-v4-pro"
backend_profile = "openai-compatible-chat-v1"

[model_routing.audio_transcript]
model = "qwen/qwen3-asr-flash-2026-02-10"
backend_profile = "hosted-audio-transcript-v1"

[model_routing.image_extract]
model = "qwen/qwen3-vl-8b-instruct"
backend_profile = "hosted-vlm-image-extract-v1"
"#,
    )?;

    assert!(load_model_routing_config_from_wendao_toml(temp.path())?.is_none());
    Ok(())
}

#[test]
fn load_wendaograph_ontology_quality_endpoint_from_wendao_toml_reads_effective_config() -> TestResult
{
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[wendaograph.ontology_read_model_quality]
base_url = "http://127.0.0.1:19091/"
timeout_seconds = 30
max_in_flight_requests = 2
"#,
    )?;

    let Some(config) =
        load_wendaograph_ontology_read_model_quality_endpoint_from_wendao_toml(temp.path())
    else {
        panic!("WendaoGraph ontology quality endpoint config should load");
    };
    assert_eq!(config.base_url, "http://127.0.0.1:19091");
    assert_eq!(config.timeout_seconds, Some(30));
    assert_eq!(config.max_in_flight_requests, Some(2));
    Ok(())
}

#[test]
fn load_episteme_registry_from_wendao_toml_accepts_path_and_url_entries() -> TestResult {
    let temp = tempfile::tempdir()?;
    fs::write(
        studio_wendao_toml_path(temp.path()),
        r#"[episteme.registries.local_domain]
path = ".data/local-episteme"
subdir = "domain"

[episteme.registries.remote_domain]
url = "https://github.com/SciML/ADTypes.jl.git"

[episteme.registries.disabled_domain]
path = ".data/disabled-episteme"
enabled = false
"#,
    )?;

    let entries = load_episteme_registry_from_wendao_toml(temp.path())?;

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].id, "disabled_domain");
    assert!(!entries[0].enabled);
    assert_eq!(
        entries[1].path.as_deref(),
        Some(std::path::Path::new(".data/local-episteme"))
    );
    assert_eq!(entries[1].subdir, std::path::PathBuf::from("domain"));
    assert_eq!(
        entries[2].url.as_deref(),
        Some("https://github.com/SciML/ADTypes.jl.git")
    );
    assert_eq!(entries[2].subdir, std::path::PathBuf::from("."));
    Ok(())
}
