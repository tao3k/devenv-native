use crate::studio::router::handlers::repo::index::repo_index_status_payload_with_diagnostics;
use xiuxian_wendao::repo_index::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};

#[tokio::test]
async fn repo_index_status_payload_recomputes_diagnostics_before_json_serialization() {
    let input = RepoIndexStatusResponse {
        total: 41,
        active: 41,
        queued: 41,
        checking: 41,
        syncing: 41,
        indexing: 41,
        ready: 41,
        unsupported: 41,
        failed: 41,
        target_concurrency: 2,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: Some("stale-current".to_string()),
        active_repo_ids: vec!["gateway-sync".to_string(), "gateway-failed".to_string()],
        repos: vec![
            RepoIndexEntryStatus {
                repo_id: "gateway-ready".to_string(),
                phase: RepoIndexPhase::Ready,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev:456".to_string()),
                updated_at: Some("2026-04-03T19:16:00Z".to_string()),
                attempt_count: 1,
            },
            RepoIndexEntryStatus {
                repo_id: "gateway-sync".to_string(),
                phase: RepoIndexPhase::Syncing,
                queue_position: None,
                last_error: None,
                last_revision: Some("rev:789".to_string()),
                updated_at: Some("2026-04-03T19:18:00Z".to_string()),
                attempt_count: 2,
            },
            RepoIndexEntryStatus {
                repo_id: "gateway-failed".to_string(),
                phase: RepoIndexPhase::Failed,
                queue_position: None,
                last_error: Some("boom".to_string()),
                last_revision: None,
                updated_at: Some("2026-04-03T19:17:00Z".to_string()),
                attempt_count: 3,
            },
        ],
    };

    let response = repo_index_status_payload_with_diagnostics(input).await;

    assert_eq!(response.total, 3);
    assert_eq!(response.active, 2);
    assert_eq!(response.queued, 0);
    assert_eq!(response.checking, 0);
    assert_eq!(response.syncing, 1);
    assert_eq!(response.indexing, 0);
    assert_eq!(response.ready, 1);
    assert_eq!(response.unsupported, 0);
    assert_eq!(response.failed, 1);
    assert_eq!(response.current_repo_id.as_deref(), Some("gateway-sync"));
    assert_eq!(
        response.active_repo_ids,
        vec!["gateway-sync".to_string(), "gateway-failed".to_string()]
    );
}
