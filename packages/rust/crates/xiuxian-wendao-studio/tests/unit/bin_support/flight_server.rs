use std::fs;

use super::{
    collect_configured_repo_content_documents, configured_repo_content_analysis,
    search_flight_grpc_web_enabled_with_lookup,
};
use xiuxian_wendao::analyzers::RegisteredRepository;

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
    fs::create_dir_all(tempdir.path().join(".cache"))?;
    fs::create_dir_all(tempdir.path().join(".git"))?;
    fs::create_dir_all(tempdir.path().join(".run"))?;
    fs::write(tempdir.path().join("docs/search.md"), "# Search\n")?;
    fs::write(tempdir.path().join("src/lib.jl"), "module Demo end\n")?;
    fs::write(tempdir.path().join("src/lib.rs"), "pub fn demo() {}\n")?;
    fs::write(tempdir.path().join("src/worker.py"), "def demo(): pass\n")?;
    fs::write(tempdir.path().join("Project.toml"), "name = \"Demo\"\n")?;
    fs::write(
        tempdir.path().join(".cache/ignored.toml"),
        "name = \"Ignored\"\n",
    )?;
    fs::write(tempdir.path().join(".git/ignored.md"), "# Ignored\n")?;
    fs::write(
        tempdir.path().join(".run/ignored.rs"),
        "pub fn ignored() {}\n",
    )?;
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

    assert_eq!(
        paths,
        vec![
            "Project.toml",
            "docs/search.md",
            "src/lib.jl",
            "src/lib.rs",
            "src/worker.py"
        ]
    );
    assert_eq!(
        languages,
        vec![
            Some("toml"),
            Some("markdown"),
            Some("julia"),
            Some("rust"),
            Some("python")
        ]
    );
    Ok(())
}

#[test]
fn configured_repo_content_analysis_projects_all_supported_documents()
-> Result<(), Box<dyn std::error::Error>> {
    let tempdir = tempfile::tempdir()?;
    fs::create_dir_all(tempdir.path().join("docs"))?;
    fs::create_dir_all(tempdir.path().join("src"))?;
    fs::write(tempdir.path().join("docs/search.md"), "# Search\n")?;
    fs::write(tempdir.path().join("src/lib.rs"), "pub fn demo() {}\n")?;
    fs::write(tempdir.path().join("wendao.toml"), "[link_graph]\n")?;

    let documents = collect_configured_repo_content_documents(tempdir.path())?;
    let repository = RegisteredRepository {
        id: "fixture".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        ..RegisteredRepository::default()
    };

    let analysis = configured_repo_content_analysis(
        &repository,
        tempdir.path(),
        Some("rev-1".to_string()),
        &documents,
    );
    let docs = analysis
        .docs
        .iter()
        .map(|doc| {
            (
                doc.doc_id.as_str(),
                doc.path.as_str(),
                doc.format.as_deref(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        docs,
        vec![
            (
                "repo:fixture:doc:docs/search.md",
                "docs/search.md",
                Some("md")
            ),
            (
                "repo:fixture:doc:src/lib.rs",
                "src/lib.rs",
                Some("reference")
            ),
            (
                "repo:fixture:doc:wendao.toml",
                "wendao.toml",
                Some("reference")
            )
        ]
    );
    Ok(())
}
