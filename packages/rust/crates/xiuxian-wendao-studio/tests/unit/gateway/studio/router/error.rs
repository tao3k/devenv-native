use axum::http::StatusCode;

use crate::studio::router::map_repo_intelligence_error;
use xiuxian_wendao::analyzers::RepoIntelligenceError;

#[test]
fn map_repo_intelligence_error_reports_missing_repo_intelligence_plugins() {
    let error =
        map_repo_intelligence_error(RepoIntelligenceError::MissingRepoIntelligencePlugins {
            repo_id: "sample".to_string(),
        });

    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
    assert_eq!(error.code(), "MISSING_REQUIRED_PLUGIN");
    assert_eq!(
        error.error.message,
        "repo `sample` does not configure any repo-intelligence plugins"
    );
}
