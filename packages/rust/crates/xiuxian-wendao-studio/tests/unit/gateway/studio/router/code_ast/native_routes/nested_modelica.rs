use std::time::Duration;

use serial_test::serial;
use tokio::time::timeout;

use crate::studio::router::handlers::analysis::load_code_ast_analysis_response;
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use super::support::{configure_repo_project, make_gateway_fixture, workspace_root};

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_nested_modelica_leaf_repository_within_timeout()
-> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed nested Modelica code-AST proof");
        return Ok(());
    }

    crate::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = workspace_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary",
    );
    if !repo_dir.is_dir() {
        eprintln!(
            "skipping process-managed nested Modelica code-AST proof; missing {}",
            repo_dir.display()
        );
        return Ok(());
    }

    let repository = RegisteredRepository {
        id: "mcl".to_string(),
        path: Some(repo_dir),
        url: Some("https://github.com/modelica/ModelicaStandardLibrary.git".to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["modelica".to_string()],
    );

    let response = timeout(
        Duration::from_secs(15),
        load_code_ast_analysis_response(
            fixture.state.as_ref(),
            "Modelica/Clocked/Types/SolverMethod.mo",
            repository.id.as_str(),
            Some(1),
        ),
    )
    .await
    .unwrap_or_else(|_| panic!("nested Modelica leaf code-AST analysis timed out"))
    .unwrap_or_else(|error| panic!("nested Modelica leaf code-AST analysis response: {error:?}"));

    assert_eq!(response.language, "modelica");
    assert_eq!(response.path, "Modelica/Clocked/Types/SolverMethod.mo");
    assert!(
        response
            .nodes
            .iter()
            .any(|node| node.label == "SolverMethod"
                && node.path.as_deref() == Some("Modelica/Clocked/Types/SolverMethod.mo")),
        "expected nested Modelica leaf node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_nested_modelica_package_repository_within_timeout()
-> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed nested Modelica package code-AST proof");
        return Ok(());
    }

    crate::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = workspace_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary",
    );
    if !repo_dir.is_dir() {
        eprintln!(
            "skipping process-managed nested Modelica package code-AST proof; missing {}",
            repo_dir.display()
        );
        return Ok(());
    }

    let repository = RegisteredRepository {
        id: "mcl".to_string(),
        path: Some(repo_dir),
        url: Some("https://github.com/modelica/ModelicaStandardLibrary.git".to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["modelica".to_string()],
    );

    let response = timeout(
        Duration::from_secs(15),
        load_code_ast_analysis_response(
            fixture.state.as_ref(),
            "Modelica/Blocks/package.mo",
            repository.id.as_str(),
            Some(1),
        ),
    )
    .await
    .unwrap_or_else(|_| panic!("nested Modelica package code-AST analysis timed out"))
    .unwrap_or_else(|error| {
        panic!("nested Modelica package code-AST analysis response: {error:?}")
    });

    assert_eq!(response.language, "modelica");
    assert_eq!(response.path, "Modelica/Blocks/package.mo");
    assert!(
        response.nodes.iter().any(|node| node.label == "Blocks"
            && node.path.as_deref() == Some("Modelica/Blocks/package.mo")),
        "expected nested Modelica package node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.attributes
                .iter()
                .any(|(key, value)| key == "restriction" && value == "package")
        }),
        "expected package retrieval atom in nested Modelica package response: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.owner_id, &atom.display_label, &atom.attributes))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_nested_modelica_package_repository_from_linked_service_within_timeout()
-> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping linked nested Modelica package code-AST proof");
        return Ok(());
    }

    crate::studio::search::handlers::tests::linked_parser_summary::ensure_linked_modelica_parser_summary_service()?;
    let fixture = make_gateway_fixture()?;
    let repo_dir = workspace_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary",
    );
    if !repo_dir.is_dir() {
        eprintln!(
            "skipping linked nested Modelica package code-AST proof; missing {}",
            repo_dir.display()
        );
        return Ok(());
    }

    let repository = RegisteredRepository {
        id: "mcl".to_string(),
        path: Some(repo_dir),
        url: Some("https://github.com/modelica/ModelicaStandardLibrary.git".to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
    };
    configure_repo_project(
        fixture.state.studio.as_ref(),
        &repository,
        vec!["modelica".to_string()],
    );

    let response = timeout(
        Duration::from_secs(35),
        load_code_ast_analysis_response(
            fixture.state.as_ref(),
            "Modelica/Clocked/Types/package.mo",
            repository.id.as_str(),
            Some(1),
        ),
    )
    .await
    .unwrap_or_else(|_| panic!("linked nested Modelica package code-AST analysis timed out"))
    .unwrap_or_else(|error| {
        panic!("linked nested Modelica package code-AST analysis response: {error:?}")
    });

    assert_eq!(response.language, "modelica");
    assert_eq!(response.path, "Modelica/Clocked/Types/package.mo");
    assert!(
        response.nodes.iter().any(|node| node.label == "Types"
            && node.path.as_deref() == Some("Modelica/Clocked/Types/package.mo")),
        "expected linked nested Modelica package node in code-AST response: {:?}",
        response
            .nodes
            .iter()
            .map(|node| (&node.label, &node.kind, &node.path))
            .collect::<Vec<_>>()
    );
    assert!(
        response.retrieval_atoms.iter().any(|atom| {
            atom.attributes
                .iter()
                .any(|(key, value)| key == "restriction" && value == "package")
        }),
        "expected package retrieval atom in linked nested Modelica package response: {:?}",
        response
            .retrieval_atoms
            .iter()
            .map(|atom| (&atom.owner_id, &atom.display_label, &atom.attributes))
            .collect::<Vec<_>>()
    );
    Ok(())
}
