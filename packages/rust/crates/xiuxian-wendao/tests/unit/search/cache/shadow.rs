use super::support::{
    SearchCorpusKind, SearchManifestRecord, SearchPublicationStorageFormat, cache_for_tests,
};

#[tokio::test]
async fn corpus_manifest_round_trips_through_test_shadow() {
    let cache = cache_for_tests();
    let record = SearchManifestRecord {
        corpus: SearchCorpusKind::KnowledgeSection,
        active_epoch: Some(7),
        schema_version: SearchCorpusKind::KnowledgeSection.schema_version(),
        storage_format: SearchPublicationStorageFormat::Parquet,
        fingerprint: Some("fingerprint".to_string()),
        row_count: Some(9),
        fragment_count: Some(1),
        build_finished_at: Some("2026-04-13T21:00:00Z".to_string()),
        updated_at: Some("2026-04-13T21:00:01Z".to_string()),
    };

    cache.set_corpus_manifest(&record).await;

    assert_eq!(
        cache
            .get_corpus_manifest(SearchCorpusKind::KnowledgeSection)
            .await,
        Some(record.clone())
    );
    assert_eq!(
        cache.get_corpus_manifest_blocking(SearchCorpusKind::KnowledgeSection),
        Some(record)
    );
}

#[tokio::test]
async fn generic_json_cache_uses_test_shadow_without_live_client() {
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct ProbePayload {
        value: String,
    }

    let cache = cache_for_tests();
    let key = "xiuxian:test:search_plane:hot_query:probe";
    let payload = ProbePayload {
        value: "cached".to_string(),
    };

    cache
        .set_json(
            key,
            crate::search::cache::SearchPlaneCacheTtl::HotQuery,
            &payload,
        )
        .await;

    let cached: Option<ProbePayload> = cache.get_json(key).await;
    assert_eq!(cached, Some(payload));
}
