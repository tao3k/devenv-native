use std::fs;

use super::{
    collect_configured_repo_content_documents, search_flight_grpc_web_enabled_with_lookup,
};

#[test]
fn search_flight_grpc_web_defaults_to_disabled() {
    assert!(!search_flight_grpc_web_enabled_with_lookup(&|_| None));
}

#[test]
fn search_flight_grpc_web_accepts_explicit_override() {
    assert!(search_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED" => Some("true".to_string()),
            _ => None,
        }
    ));
    assert!(!search_flight_grpc_web_enabled_with_lookup(
        &|key| match key {
            "XIUXIAN_WENDAO_SEARCH_FLIGHT_GRPC_WEB_ENABLED" => Some("false".to_string()),
            _ => None,
        }
    ));
}

#[test]
fn configured_repo_content_bootstrap_collects_supported_text_files()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    fs::create_dir_all(tempdir.path().join("docs"))?;
    fs::create_dir_all(tempdir.path().join("src"))?;
    fs::create_dir_all(tempdir.path().join(".git"))?;
    fs::write(tempdir.path().join("docs/search.md"), "# Search\n")?;
    fs::write(tempdir.path().join("src/lib.jl"), "module Demo end\n")?;
    fs::write(tempdir.path().join("Project.toml"), "name = \"Demo\"\n")?;
    fs::write(tempdir.path().join(".git/ignored.md"), "# Ignored\n")?;
    fs::write(tempdir.path().join("image.png"), "not indexed\n")?;

    let documents = collect_configured_repo_content_documents(tempdir.path())?;
    let paths = documents
        .iter()
        .map(|document| document.path.as_str())
        .collect::<Vec<_>>();
    let languages = documents
        .iter()
        .map(|document| document.language.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(paths, vec!["Project.toml", "docs/search.md", "src/lib.jl"]);
    assert_eq!(
        languages,
        vec![Some("toml"), Some("markdown"), Some("julia")]
    );
    Ok(())
}
