use std::fs;
use std::path::Path;
use std::sync::Arc;

use serde_json::json;

use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use crate::gateway::studio::router::handlers::analysis::service::load_code_ast_analysis_response;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::search::handlers::tests::linked_parser_summary::{
    ensure_linked_julia_parser_summary_service, ensure_linked_modelica_parser_summary_service,
};
use crate::gateway::studio::test_support::{
    assert_studio_json_snapshot, commit_all, init_git_repository,
};
use crate::gateway::studio::types::{CodeAstAnalysisResponse, UiConfig, UiRepoProjectConfig};
use serial_test::serial;

#[tokio::test]
#[serial(julia_live)]
async fn load_code_ast_analysis_response_supports_plain_julia_plugin_repository()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
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
    ensure_linked_modelica_parser_summary_service()?;
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
    ensure_linked_modelica_parser_summary_service()?;
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
                && matches!(
                    node.kind,
                    crate::gateway::studio::types::CodeAstNodeKind::Function
                )
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
                && matches!(
                    node.kind,
                    crate::gateway::studio::types::CodeAstNodeKind::Module
                )
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

struct GatewayFixture {
    state: Arc<GatewayState>,
    temp_dir: tempfile::TempDir,
}

fn make_gateway_fixture() -> Result<GatewayFixture, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let search_plane_root = temp_dir.path().join("search-plane");
    let studio = StudioState::new_with_bootstrap_ui_config_and_search_plane_root(
        Arc::new(crate::analyzers::bootstrap_builtin_registry()?),
        search_plane_root,
    );
    Ok(GatewayFixture {
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio),
        }),
        temp_dir,
    })
}

fn configure_repo_project(
    studio: &StudioState,
    repository: &RegisteredRepository,
    plugins: Vec<String>,
) {
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: repository.id.clone(),
            root: repository
                .path
                .as_ref()
                .map(|path| path.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins,
        }],
    });
}

fn create_sample_julia_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            "name = \"{package_name}\"\nuuid = \"12345678-1234-1234-1234-123456789abc\"\nversion = \"0.1.0\"\n"
        ),
    )?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!(
            r"module {package_name}

export solve, Problem

struct Problem
    x::Int
end

function solve(problem::Problem)
    problem.x
end

end
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn create_sample_modelica_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("Controllers"))?;
    fs::write(
        repo_dir.join("package.mo"),
        format!(
            r"within ;
package {package_name}
end {package_name};
"
        ),
    )?;
    fs::write(
        repo_dir.join("Controllers/PI.mo"),
        format!(
            r"within {package_name}.Controllers;
model PI
  parameter Real k = 1;
  parameter Real Ti = 0.1;
end PI;
"
        ),
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn create_import_backed_modelica_repo(
    base: &Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join("modelica_import_backed");
    fs::create_dir_all(repo_dir.join("Modelica/Blocks"))?;
    fs::write(
        repo_dir.join("Modelica/package.mo"),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    fs::write(
        repo_dir.join("Modelica/Blocks/package.mo"),
        "within Modelica;\npackage Blocks\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\nend Blocks;\n",
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn create_sample_rust_repo(
    base: &Path,
    package_name: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            package_name.to_ascii_lowercase()
        ),
    )?;
    fs::write(
        repo_dir.join("src/lib.rs"),
        r"pub struct Dataset;

fn scan_rows(dataset: &Dataset) {
    let _ = dataset;
}
",
    )?;
    initialize_git_fixture(repo_dir.as_path());
    Ok(repo_dir)
}

fn initialize_git_fixture(repo_dir: &Path) {
    init_git_repository(repo_dir);
    commit_all(repo_dir, "seed fixture");
}
