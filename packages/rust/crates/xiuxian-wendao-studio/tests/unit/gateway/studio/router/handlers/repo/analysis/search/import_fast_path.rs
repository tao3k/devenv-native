use super::support::{sample_import_gateway_fixture, sample_repo_entity_gateway_fixture};
use super::{Arc, assert_studio_json_snapshot, run_repo_import_search};

#[tokio::test]
async fn repo_import_search_uses_repo_entity_fast_path_when_publication_ready() {
    let fixture = sample_repo_entity_gateway_fixture("xiuxian:test:repo_import_fast_path").await;

    let result = run_repo_import_search(
        Arc::clone(&fixture.state),
        "alpha/repo".to_string(),
        Some("SciMLBase".to_string()),
        Some("BaseModelica".to_string()),
        5,
    )
    .await
    .unwrap_or_else(|error| {
        panic!("repo import search should resolve through repo entity fast path: {error:?}")
    });

    assert_eq!(result.imports.len(), 1);
    assert_eq!(result.imports[0].target_package, "SciMLBase");
    assert_eq!(result.imports[0].source_module, "BaseModelica");
}

#[tokio::test]
async fn repo_import_search_payload_snapshot() {
    let fixture = sample_import_gateway_fixture("xiuxian:test:repo_import_search_payload");

    let result = run_repo_import_search(
        Arc::clone(&fixture.state),
        "sciml/imports".to_string(),
        Some("SciMLBase".to_string()),
        None,
        10,
    )
    .await
    .unwrap_or_else(|error| panic!("repo import search should resolve: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_import_search_payload", result);
}
