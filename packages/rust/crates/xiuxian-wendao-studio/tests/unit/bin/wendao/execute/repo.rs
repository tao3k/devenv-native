use super::{DocCoverageQuery, RepoOverviewQuery};

#[test]
fn repo_overview_query_preserves_repo_id() {
    let query = RepoOverviewQuery {
        repo_id: "sciml".to_string(),
    };
    assert_eq!(query.repo_id, "sciml");
}

#[test]
fn doc_coverage_query_preserves_optional_module_scope() {
    let query = DocCoverageQuery {
        repo_id: "sciml".to_string(),
        module_id: Some("BaseModelica".to_string()),
    };
    assert_eq!(query.repo_id, "sciml");
    assert_eq!(query.module_id.as_deref(), Some("BaseModelica"));
}
