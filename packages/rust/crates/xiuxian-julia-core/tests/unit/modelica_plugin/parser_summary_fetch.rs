use std::fs;

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::{
    fetch_modelica_parser_file_summary_blocking_for_repository,
    modelica_file_summary_blocking_timeout_secs_for_repository,
    shared_modelica_parser_summary_runtime_identity_for_tests,
};
use crate::julia_plugin_test_support::common::{
    ensure_linked_modelica_parser_summary_service, repo_root,
    skip_linked_modelica_parser_summary_service_if_unavailable,
};

fn parser_summary_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
        ..RegisteredRepository::default()
    }
}

fn parser_summary_repository_with_timeout(timeout_secs: u64) -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-modelica-timeout".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:41081",
                    "file_summary": {
                        "timeout_secs": timeout_secs
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    }
}

#[test]
fn blocking_fetch_timeout_follows_transport_timeout_without_hidden_cap() {
    let repository = parser_summary_repository_with_timeout(60);

    let timeout_secs = modelica_file_summary_blocking_timeout_secs_for_repository(&repository)
        .unwrap_or_else(|error| panic!("expected file-summary timeout to resolve: {error}"));

    assert_eq!(timeout_secs, 60);
}

#[test]
#[serial_test::serial(modelica_live)]
fn blocking_fetch_reuses_shared_runtime_and_returns_summary_from_linked_service()
-> Result<(), Box<dyn std::error::Error>> {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let repository = parser_summary_repository();
    let runtime_before = shared_modelica_parser_summary_runtime_identity_for_tests()?;
    let source = r"
within Demo;
model GainHolder
  parameter Real gain = 1;
end GainHolder;
";

    let first = fetch_modelica_parser_file_summary_blocking_for_repository(
        &repository,
        "Demo/GainHolder.mo",
        source,
    )?;
    let runtime_after_first = shared_modelica_parser_summary_runtime_identity_for_tests()?;
    let second = fetch_modelica_parser_file_summary_blocking_for_repository(
        &repository,
        "Demo/GainHolder.mo",
        source,
    )?;
    let runtime_after_second = shared_modelica_parser_summary_runtime_identity_for_tests()?;

    assert_eq!(runtime_before, runtime_after_first);
    assert_eq!(runtime_after_first, runtime_after_second);
    assert_eq!(first.class_name.as_deref(), Some("GainHolder"));
    assert_eq!(second.class_name.as_deref(), Some("GainHolder"));
    assert!(
        first
            .declarations
            .iter()
            .any(|declaration| declaration.name == "GainHolder"),
        "expected GainHolder declaration in first summary: {:?}",
        first.declarations,
    );
    assert!(
        second
            .declarations
            .iter()
            .any(|declaration| declaration.name == "GainHolder"),
        "expected GainHolder declaration in second summary: {:?}",
        second.declarations,
    );

    Ok(())
}

#[test]
#[serial_test::serial(modelica_large_live)]
fn blocking_fetch_supports_large_modelica_standard_library_package_from_linked_service()
-> Result<(), Box<dyn std::error::Error>> {
    if skip_linked_modelica_parser_summary_service_if_unavailable() {
        return Ok(());
    }
    ensure_linked_modelica_parser_summary_service()?;
    let source_path = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary/Modelica/Mechanics/MultiBody/package.mo",
    );
    if !source_path.is_file() {
        eprintln!(
            "skipping linked Modelica parser-summary large-package proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let source_text = fs::read_to_string(&source_path)?;
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        &parser_summary_repository(),
        "Modelica/Mechanics/MultiBody/package.mo",
        &source_text,
    )?;

    assert_eq!(summary.class_name.as_deref(), Some("MultiBody"));
    assert!(
        summary
            .declarations
            .iter()
            .any(|declaration| declaration.name == "World"),
        "expected World declaration in MultiBody summary: {:?}",
        summary.declarations,
    );
    assert!(
        summary
            .declarations
            .iter()
            .any(|declaration| declaration.name == "gravityAcceleration"),
        "expected representative MultiBody member declaration: {:?}",
        summary.declarations,
    );
    assert!(
        summary.declarations.len() >= 2,
        "expected substantial MultiBody declaration surface, got {}",
        summary.declarations.len(),
    );

    Ok(())
}
