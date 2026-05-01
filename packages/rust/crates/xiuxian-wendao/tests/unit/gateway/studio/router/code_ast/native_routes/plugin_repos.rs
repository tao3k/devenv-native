use serde_json::json;
use serial_test::serial;

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use crate::gateway::studio::router::handlers::analysis::load_code_ast_analysis_response;
use crate::gateway::studio::test_support::assert_studio_json_snapshot;
use crate::gateway::studio::types::CodeAstAnalysisResponse;

use super::support::{
    configure_repo_project, create_import_backed_modelica_repo, create_sample_julia_repo,
    create_sample_modelica_repo, make_gateway_fixture,
};

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_plain_julia_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    crate::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = create_sample_julia_repo(fixture.temp_dir.path(), "CodeAstJulia")?;
    let repository = RegisteredRepository {
        id: "julia-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["julia".to_string()],
    );

    let response: CodeAstAnalysisResponse = load_code_ast_analysis_response(
        fixture.state.as_ref(),
        "src/CodeAstJulia.jl",
        repository.id.as_str(),
        Some(8),
    )
    .await
    .unwrap_or_else(|error| panic!("Julia code-AST analysis response: {error:?}"));

    assert_eq!(response.language, "julia");
    assert_eq!(response.path, "src/CodeAstJulia.jl");
    assert!(
        response.nodes.iter().any(|node| node.label == "solve"
            && matches!(
                node.kind,
                crate::gateway::studio::types::CodeAstNodeKind::Function
            )
            && node.path.as_deref() == Some("src/CodeAstJulia.jl")),
        "expected Julia function node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.chunk_id.starts_with("ast:src-codeastjulia-jl:")
                && atom.owner_id.contains(":symbol:")
                && matches!(
                    atom.surface,
                    Some(crate::gateway::studio::types::CodeAstRetrievalAtomScope::Declaration)
                )
        }),
        "expected Julia declaration retrieval atoms: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.owner_id, &atom.chunk_id, &atom.surface))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_plain_modelica_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    crate::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = create_sample_modelica_repo(fixture.temp_dir.path(), "CodeAstModelica")?;
    let repository = RegisteredRepository {
        id: "modelica-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["modelica".to_string()],
    );

    let response: CodeAstAnalysisResponse = load_code_ast_analysis_response(
        fixture.state.as_ref(),
        "Controllers/PI.mo",
        repository.id.as_str(),
        Some(2),
    )
    .await
    .unwrap_or_else(|error| panic!("Modelica code-AST analysis response: {error:?}"));

    assert_eq!(response.language, "modelica");
    assert_eq!(response.path, "Controllers/PI.mo");
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.label == "PI" && node.path.as_deref() == Some("Controllers/PI.mo")),
        "expected Modelica node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.chunk_id.starts_with("ast:controllers-pi-mo:")
                && atom.owner_id.contains(":symbol:")
        }),
        "expected Modelica retrieval atoms: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.owner_id, &atom.chunk_id, &atom.surface))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_import_backed_modelica_package_repository()
-> Result<(), Box<dyn std::error::Error>> {
    crate::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = create_import_backed_modelica_repo(fixture.temp_dir.path())?;
    let repository = RegisteredRepository {
        id: "modelica-import-live".to_string(),
        path: Some(repo_dir.clone()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["modelica".to_string()],
    );
    let registry = crate::analyzers::bootstrap_builtin_registry()?;
    crate::analyzers::analyze_registered_repository_with_registry(
        &repository,
        fixture.temp_dir.path(),
        &registry,
    )?;

    let response: CodeAstAnalysisResponse = load_code_ast_analysis_response(
        fixture.state.as_ref(),
        "Modelica/Blocks/package.mo",
        repository.id.as_str(),
        Some(3),
    )
    .await
    .unwrap_or_else(|error| panic!("Modelica import-backed code-AST analysis response: {error:?}"));

    let payload = json!({
        "path": response.path,
        "language": response.language,
        "import_nodes": response
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, crate::gateway::studio::types::CodeAstNodeKind::ExternalSymbol))
            .map(|node| json!({
                "id": node.id,
                "label": node.label,
                "path": node.path,
                "line_start": node.line_start,
                "line_end": node.line_end,
            }))
            .collect::<Vec<_>>(),
        "import_edges": response
            .edges
            .iter()
            .filter(|edge| matches!(edge.kind, crate::gateway::studio::types::CodeAstEdgeKind::Imports))
            .map(|edge| json!({
                "source_id": edge.source_id,
                "target_id": edge.target_id,
            }))
            .collect::<Vec<_>>(),
        "import_atoms": response
            .retrieval_atoms
            .iter()
            .filter(|atom| atom.semantic_type.starts_with("import"))
            .map(|atom| json!({
                "owner_id": atom.owner_id,
                "display_label": atom.display_label,
                "excerpt": atom.excerpt,
                "line_start": atom.line_start,
                "attributes": atom.attributes,
            }))
            .collect::<Vec<_>>(),
    });
    assert_studio_json_snapshot(
        "analysis_code_ast_modelica_import_backed_package_payload",
        payload,
    );
    Ok(())
}
