use std::fs;
use std::time::{Duration, Instant};

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use crate::julia_plugin_test_support::common::{
    ensure_linked_modelica_parser_summary_service, repo_root,
};
use crate::{
    fetch_modelica_ast_query_analysis_blocking_for_repository,
    modelica_plugin::ast_query::fetch::modelica_ast_query_blocking_timeout_secs_for_repository,
};

fn ast_query_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-modelica".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
        ..RegisteredRepository::default()
    }
}

fn ast_query_repository_with_timeout(timeout_secs: u64) -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-modelica-ast-timeout".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: json!({
                "parser_summary_transport": {
                    "base_url": "http://127.0.0.1:41081",
                    "ast_query": {
                        "timeout_secs": timeout_secs
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    }
}

#[test]
fn blocking_ast_query_timeout_follows_transport_timeout_without_hidden_cap() {
    let repository = ast_query_repository_with_timeout(60);

    let timeout_secs = modelica_ast_query_blocking_timeout_secs_for_repository(&repository)
        .unwrap_or_else(|error| panic!("expected ast-query timeout to resolve: {error}"));

    assert_eq!(timeout_secs, 60);
}

#[test]
#[serial_test::serial(modelica_live)]
fn blocking_ast_query_analysis_returns_lightweight_package_surface_from_linked_service()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let source = r#"
within Demo;
package Blocks
  import Modelica.Units.SI;

  package Examples
    model GainHolder
      parameter SI.Angle driveAngle = 1.0;
    end GainHolder;
  end Examples;
end Blocks;
"#;

    let analysis = fetch_modelica_ast_query_analysis_blocking_for_repository(
        &ast_query_repository(),
        "Demo/Blocks/package.mo",
        source,
    )?;

    assert!(
        analysis
            .modules
            .iter()
            .any(|module| module.qualified_name == "Blocks"
                && module.path == "Demo/Blocks/package.mo"),
        "expected package module in ast-query analysis: {:?}",
        analysis.modules,
    );
    assert!(
        analysis.imports.iter().any(|import| {
            import.source_module == "Modelica.Units.SI"
                && import
                    .attributes
                    .get("dependency_form")
                    .is_some_and(|value| value == "qualified_import")
        }),
        "expected import in ast-query analysis: {:?}",
        analysis.imports,
    );
    assert!(
        analysis.symbols.iter().any(|symbol| {
            symbol.name == "Blocks"
                && symbol
                    .attributes
                    .get("restriction")
                    .is_some_and(|value| value == "package")
        }),
        "expected package symbol in ast-query analysis: {:?}",
        analysis.symbols,
    );
    assert!(
        analysis.symbols.iter().any(|symbol| {
            symbol.name == "driveAngle"
                && symbol
                    .attributes
                    .get("component_kind")
                    .is_some_and(|value| value == "parameter")
        }),
        "expected component symbol in ast-query analysis: {:?}",
        analysis.symbols,
    );

    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn blocking_ast_query_analysis_supports_real_modelica_standard_library_package_within_budget()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_modelica_parser_summary_service()?;
    let source_path = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary/Modelica/Blocks/package.mo",
    );
    if !source_path.is_file() {
        eprintln!(
            "skipping linked Modelica ast-query large-package proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let source_text = fs::read_to_string(&source_path)?;
    let started_at = Instant::now();
    let analysis = fetch_modelica_ast_query_analysis_blocking_for_repository(
        &ast_query_repository(),
        "Modelica/Blocks/package.mo",
        &source_text,
    )?;
    let elapsed = started_at.elapsed();

    assert!(
        elapsed < Duration::from_secs(15),
        "expected real Modelica package ast-query fetch to stay below 15s, got {:?}",
        elapsed,
    );
    assert!(
        analysis.symbols.iter().any(|symbol| symbol.name == "Blocks"
            && symbol
                .attributes
                .get("restriction")
                .is_some_and(|value| value == "package")),
        "expected Blocks package symbol in ast-query analysis: {:?}",
        analysis.symbols,
    );
    assert!(
        analysis
            .imports
            .iter()
            .any(|import| import.source_module == "Modelica.Units.SI"),
        "expected Modelica.Units.SI import in ast-query analysis: {:?}",
        analysis.imports,
    );

    Ok(())
}
