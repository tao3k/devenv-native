use serde_json::json;

use crate::studio::router::handlers::analysis::load_code_ast_analysis_response;
use crate::studio::test_support::assert_studio_json_snapshot;
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use super::support::{configure_repo_project, create_sample_rust_repo, make_gateway_fixture};

#[tokio::test]
async fn load_code_ast_analysis_response_supports_search_only_ast_grep_rust_repository()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = make_gateway_fixture()?;
    let repo_dir = create_sample_rust_repo(fixture.temp_dir.path(), "SearchRust")?;
    let repository = RegisteredRepository {
        id: "lance".to_string(),
        path: Some(repo_dir.clone()),
        url: Some("https://github.com/lance-format/lance".to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["ast-grep".to_string()],
    );

    let response = load_code_ast_analysis_response(
        fixture.state.as_ref(),
        "lance/src/lib.rs",
        repository.id.as_str(),
        Some(3),
    )
    .await
    .unwrap_or_else(|error| panic!("search-only Rust code-AST analysis response: {error:?}"));

    assert_eq!(response.language, "rust");
    assert_eq!(response.path, "lance/src/lib.rs");
    assert!(
        response.nodes.iter().any(|node| {
            node.label == "scan_rows"
                && matches!(node.kind, crate::contracts::CodeAstNodeKind::Function)
                && node.path.as_deref() == Some("src/lib.rs")
        }),
        "expected generic Rust function node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.display_label.as_deref() == Some("Declaration Rail · scan_rows")
                && atom.semantic_type == "function"
                && atom
                    .attributes
                    .iter()
                    .any(|(key, value)| key == "analysis_mode" && value == "ast-grep")
        }),
        "expected ast-grep retrieval atoms for Rust preview: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.display_label, &atom.semantic_type, &atom.attributes))
            .collect::<Vec<_>>()
    );
    assert!(response.focus_node_id.is_some());

    let payload = json!({
        "repo_id": response.repo_id,
        "path": response.path,
        "language": response.language,
        "nodes": response
            .nodes
            .iter()
            .map(|node| json!({
                "id": node.id,
                "label": node.label,
                "kind": node.kind,
                "path": node.path,
                "line_start": node.line_start,
                "line_end": node.line_end,
            }))
            .collect::<Vec<_>>(),
        "retrieval_atoms": response
            .retrieval_atoms
            .iter()
            .map(|atom| json!({
                "owner_id": atom.owner_id,
                "semantic_type": atom.semantic_type,
                "display_label": atom.display_label,
                "excerpt": atom.excerpt,
                "line_start": atom.line_start,
                "line_end": atom.line_end,
                "attributes": atom.attributes,
            }))
            .collect::<Vec<_>>(),
        "focus_node_id": response.focus_node_id,
    });
    assert_studio_json_snapshot(
        "analysis_code_ast_search_only_ast_grep_rust_payload",
        payload,
    );
    Ok(())
}

#[tokio::test]
async fn load_code_ast_analysis_response_supports_search_only_ast_grep_toml_repository()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = make_gateway_fixture()?;
    let repo_dir = create_sample_rust_repo(fixture.temp_dir.path(), "SearchRust")?;
    let repository = RegisteredRepository {
        id: "lance".to_string(),
        path: Some(repo_dir.clone()),
        url: Some("https://github.com/lance-format/lance".to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["ast-grep".to_string()],
    );

    let response = load_code_ast_analysis_response(
        fixture.state.as_ref(),
        "lance/Cargo.toml",
        repository.id.as_str(),
        Some(2),
    )
    .await
    .unwrap_or_else(|error| panic!("search-only TOML code-AST analysis response: {error:?}"));

    assert_eq!(response.language, "toml");
    assert_eq!(response.path, "lance/Cargo.toml");
    assert!(
        response.nodes.iter().any(|node| {
            node.label == "package"
                && matches!(node.kind, crate::contracts::CodeAstNodeKind::Module)
                && node.path.as_deref() == Some("Cargo.toml")
        }),
        "expected generic TOML table node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.display_label.as_deref() == Some("Declaration Rail · package")
                && atom.semantic_type == "module"
                && atom
                    .attributes
                    .iter()
                    .any(|(key, value)| key == "language" && value == "toml")
        }),
        "expected ast-grep retrieval atoms for TOML preview: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.display_label, &atom.semantic_type, &atom.attributes))
            .collect::<Vec<_>>()
    );

    let payload = json!({
        "repo_id": response.repo_id,
        "path": response.path,
        "language": response.language,
        "nodes": response
            .nodes
            .iter()
            .map(|node| json!({
                "id": node.id,
                "label": node.label,
                "kind": node.kind,
                "path": node.path,
                "line_start": node.line_start,
                "line_end": node.line_end,
            }))
            .collect::<Vec<_>>(),
        "retrieval_atoms": response
            .retrieval_atoms
            .iter()
            .map(|atom| json!({
                "owner_id": atom.owner_id,
                "semantic_type": atom.semantic_type,
                "display_label": atom.display_label,
                "excerpt": atom.excerpt,
                "line_start": atom.line_start,
                "line_end": atom.line_end,
                "attributes": atom.attributes,
            }))
            .collect::<Vec<_>>(),
        "focus_node_id": response.focus_node_id,
    });
    assert_studio_json_snapshot(
        "analysis_code_ast_search_only_ast_grep_toml_payload",
        payload,
    );
    Ok(())
}
