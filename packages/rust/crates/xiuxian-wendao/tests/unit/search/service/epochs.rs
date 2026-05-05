use crate::repo_index::{RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::SearchCorpusKind;
use crate::search::service::helpers::{repo_corpus_active_epoch, repo_corpus_staging_epoch};
use crate::search::service::tests::support::repo_status_entry;

#[test]
fn repo_corpus_active_epoch_is_stable_across_publication_order() {
    let first = repo_corpus_active_epoch(SearchCorpusKind::RepoEntity, &[22, 11]);
    let second = repo_corpus_active_epoch(SearchCorpusKind::RepoEntity, &[11, 22]);

    assert_eq!(first, second);
}

#[test]
fn repo_corpus_active_epoch_is_stable_for_duplicate_publication_epochs() {
    let first = repo_corpus_active_epoch(SearchCorpusKind::RepoEntity, &[11, 22, 22]);
    let second = repo_corpus_active_epoch(SearchCorpusKind::RepoEntity, &[11, 22]);

    assert_eq!(first, second);
}

#[test]
fn repo_corpus_staging_epoch_tracks_refresh_state() {
    let active_epoch = Some(42);
    let idle = repo_corpus_staging_epoch(
        SearchCorpusKind::RepoContentChunk,
        &RepoIndexStatusResponse {
            total: 1,
            active: 0,
            queued: 0,
            checking: 0,
            syncing: 0,
            indexing: 0,
            ready: 1,
            unsupported: 0,
            failed: 0,
            target_concurrency: 1,
            max_concurrency: 1,
            sync_concurrency_limit: 1,
            current_repo_id: None,
            active_repo_ids: Vec::new(),
            repos: vec![repo_status_entry("alpha/repo", RepoIndexPhase::Ready)],
        }
        .repos,
        active_epoch,
    );
    let refreshing = repo_corpus_staging_epoch(
        SearchCorpusKind::RepoContentChunk,
        &RepoIndexStatusResponse {
            total: 1,
            active: 1,
            queued: 0,
            checking: 0,
            syncing: 0,
            indexing: 1,
            ready: 0,
            unsupported: 0,
            failed: 0,
            target_concurrency: 1,
            max_concurrency: 1,
            sync_concurrency_limit: 1,
            current_repo_id: Some("alpha/repo".to_string()),
            active_repo_ids: vec!["alpha/repo".to_string()],
            repos: vec![repo_status_entry("alpha/repo", RepoIndexPhase::Indexing)],
        }
        .repos,
        active_epoch,
    );

    assert!(idle.is_none());
    assert!(refreshing.is_some());
    assert_ne!(refreshing, active_epoch);
}
