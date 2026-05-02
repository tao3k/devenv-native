use super::support::sample_search_analysis;
use crate::analyzers::ImportSearchQuery;
use crate::analyzers::service::search::build_import_search;
use crate::gateway::studio::test_support::assert_wendao_json_snapshot;

#[test]
fn import_search_snapshot_matches_package_and_module_filters() {
    let analysis = sample_search_analysis("import-snapshot");
    let result = build_import_search(
        &ImportSearchQuery {
            repo_id: "import-snapshot".to_string(),
            package: Some("SciMLBase".to_string()),
            module: Some("BaseModelica".to_string()),
            limit: 10,
        },
        &analysis,
    );

    assert_wendao_json_snapshot("search_plane_import_search_results", result);
}
