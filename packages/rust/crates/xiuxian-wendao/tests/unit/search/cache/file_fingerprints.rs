use std::collections::BTreeMap;

use super::support::{
    SearchCorpusKind, SearchPlaneFileFingerprintScope, cache_for_tests, sample_file_fingerprint,
};

#[tokio::test]
async fn file_fingerprints_round_trip_through_unified_scope_api() {
    let cache = cache_for_tests();
    let corpus_fingerprints = BTreeMap::from([(
        "docs/index.md".to_string(),
        sample_file_fingerprint("docs/index.md", "docs", 12, 34),
    )]);
    let repo_fingerprints = BTreeMap::from([(
        "src/lib.rs".to_string(),
        sample_file_fingerprint("src/lib.rs", "src", 56, 78),
    )]);

    cache
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::corpus(SearchCorpusKind::KnowledgeSection),
            &corpus_fingerprints,
        )
        .await;
    cache
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
            ),
            &repo_fingerprints,
        )
        .await;

    assert_eq!(
        cache
            .get_file_fingerprints(SearchPlaneFileFingerprintScope::corpus(
                SearchCorpusKind::KnowledgeSection,
            ))
            .await,
        Some(corpus_fingerprints)
    );
    assert_eq!(
        cache
            .get_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
            ))
            .await,
        Some(repo_fingerprints)
    );
}

#[tokio::test]
async fn delete_file_fingerprints_clears_repo_scope_entries() {
    let cache = cache_for_tests();
    let repo_fingerprints = BTreeMap::from([(
        "src/lib.rs".to_string(),
        sample_file_fingerprint("src/lib.rs", "src", 56, 78),
    )]);

    cache
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
            ),
            &repo_fingerprints,
        )
        .await;
    assert!(
        cache
            .get_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
            ))
            .await
            .is_some()
    );

    cache
        .delete_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoContentChunk,
            "alpha/repo",
        ))
        .await;

    assert!(
        cache
            .get_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                "alpha/repo",
            ))
            .await
            .is_none()
    );
}
