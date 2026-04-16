use super::support::{SearchCorpusKind, cache_for_tests, required_cache_key};

#[test]
fn autocomplete_key_is_stable_for_epoch_prefix_and_limit() {
    let cache = cache_for_tests();
    let key = required_cache_key(
        cache.autocomplete_cache_key(" Alpha Handler ", 8, 7),
        "autocomplete key",
    );
    assert_eq!(
        key,
        required_cache_key(
            cache.autocomplete_cache_key("alpha    handler", 8, 7),
            "stable autocomplete key",
        )
    );
    assert_ne!(
        key,
        required_cache_key(
            cache.autocomplete_cache_key("alpha handler", 8, 8),
            "epoch-specific autocomplete key",
        )
    );
}

#[test]
fn search_query_key_tracks_scope_epochs_and_query_shape() {
    let cache = cache_for_tests();
    let key = required_cache_key(
        cache.search_query_cache_key(
            "intent",
            &[
                (SearchCorpusKind::KnowledgeSection, 3),
                (SearchCorpusKind::LocalSymbol, 11),
            ],
            "  alpha_handler  ",
            10,
            Some("semantic_lookup"),
            None,
        ),
        "search query key",
    );
    assert_eq!(
        key,
        required_cache_key(
            cache.search_query_cache_key(
                "intent",
                &[
                    (SearchCorpusKind::KnowledgeSection, 3),
                    (SearchCorpusKind::LocalSymbol, 11),
                ],
                "alpha_handler",
                10,
                Some("semantic_lookup"),
                None,
            ),
            "stable search query key",
        )
    );
    assert_ne!(
        key,
        required_cache_key(
            cache.search_query_cache_key(
                "intent",
                &[
                    (SearchCorpusKind::KnowledgeSection, 3),
                    (SearchCorpusKind::LocalSymbol, 12),
                ],
                "alpha_handler",
                10,
                Some("semantic_lookup"),
                None,
            ),
            "epoch-specific search query key",
        )
    );
}

#[test]
fn search_query_key_tracks_repo_versions_and_sorts_components() {
    let cache = cache_for_tests();
    let key = required_cache_key(
        cache.search_query_cache_key_from_versions(
            "intent_code",
            &[
                "repo_entity:schema:1:repo:alpha:phase:ready:revision:abc:updated:2026-03-23t08:00:00z"
                    .to_string(),
                "knowledge_section:schema:1:epoch:3".to_string(),
                "repo_content_chunk:schema:1:repo:alpha:phase:ready:revision:abc:updated:2026-03-23t08:00:00z"
                    .to_string(),
            ],
            " lang:julia reexport ",
            10,
            Some("debug_lookup"),
            Some("alpha"),
        ),
        "repo search query key",
    );
    assert_eq!(
        key,
        required_cache_key(
            cache.search_query_cache_key_from_versions(
                "intent_code",
                &[
                    "repo_content_chunk:schema:1:repo:alpha:phase:ready:revision:abc:updated:2026-03-23t08:00:00z"
                        .to_string(),
                    "knowledge_section:schema:1:epoch:3".to_string(),
                    "repo_entity:schema:1:repo:alpha:phase:ready:revision:abc:updated:2026-03-23t08:00:00z"
                        .to_string(),
                ],
                "lang:julia   reexport",
                10,
                Some("debug_lookup"),
                Some("alpha"),
            ),
            "stable repo search query key",
        )
    );
    assert_ne!(
        key,
        required_cache_key(
            cache.search_query_cache_key_from_versions(
                "intent_code",
                &[
                    "repo_entity:schema:1:repo:alpha:phase:ready:revision:def:updated:2026-03-23t09:00:00z"
                        .to_string(),
                    "knowledge_section:schema:1:epoch:3".to_string(),
                    "repo_content_chunk:schema:1:repo:alpha:phase:ready:revision:def:updated:2026-03-23t09:00:00z"
                        .to_string(),
                ],
                "lang:julia reexport",
                10,
                Some("debug_lookup"),
                Some("alpha"),
            ),
            "repo-specific search query key",
        )
    );
}

#[test]
fn disabled_cache_skips_key_generation() {
    let cache = crate::search::cache::SearchPlaneCache::disabled(
        crate::search::SearchManifestKeyspace::new("xiuxian:test"),
    );
    assert!(cache.autocomplete_cache_key("alpha", 8, 1).is_none());
    assert!(
        cache
            .search_query_cache_key(
                "knowledge",
                &[(SearchCorpusKind::KnowledgeSection, 1)],
                "alpha",
                10,
                None,
                None,
            )
            .is_none()
    );
}
