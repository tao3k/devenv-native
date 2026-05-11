use super::runtime::{
    resolve_valkey_analysis_cache_runtime_with_lookup,
    resolve_valkey_analysis_cache_runtime_with_settings_and_lookup_for_tests,
};
use super::storage::{
    decode_analysis_payload, decode_analysis_payload_for_revision, decode_search_query_payload,
    encode_analysis_payload, encode_search_query_payload, valkey_analysis_key,
    valkey_analysis_revision_key, valkey_search_query_key,
};
use super::{RepositoryAnalysisValkeyScope, RepositorySearchQueryValkeyScope, ValkeyAnalysisCache};
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::cache::{RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey};
use crate::search::FuzzySearchOptions;
use serde_yaml::Value;

fn sample_cache_key(repo_id: &str) -> RepositoryAnalysisCacheKey {
    RepositoryAnalysisCacheKey {
        repo_id: repo_id.to_string().into(),
        checkout_root: format!("/virtual/{repo_id}"),
        analysis_identity: format!("analysis:{repo_id}"),
        checkout_revision: Some("rev-1".to_string()),
        mirror_revision: Some("mirror-1".to_string()),
        tracking_revision: Some("tracking-1".to_string()),
        plugin_ids: vec!["plugin-a".to_string()],
    }
}

fn sample_query_cache_key(repo_id: &str) -> RepositorySearchQueryCacheKey {
    RepositorySearchQueryCacheKey::new(
        &sample_cache_key(repo_id),
        "repo.projected-page-search",
        "solve",
        Some("reference".to_string()),
        FuzzySearchOptions::document_search(),
        5,
    )
}

fn settings_from_yaml(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).unwrap_or_else(|error| panic!("settings yaml: {error}"))
}

#[test]
fn runtime_resolution_uses_first_non_empty_url_and_normalized_prefix() {
    let runtime = resolve_valkey_analysis_cache_runtime_with_lookup(&|name| match name {
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_URL" => Some(" redis://127.0.0.1/ ".to_string()),
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_KEY_PREFIX" => {
            Some("  xiuxian:test:repo-analysis  ".to_string())
        }
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_TTL_SECS" => Some("3600".to_string()),
        _ => None,
    })
    .unwrap_or_else(|error| panic!("runtime resolution should succeed: {error}"))
    .unwrap_or_else(|| panic!("runtime should exist"));

    assert_eq!(runtime.key_prefix, "xiuxian:test:repo-analysis");
    assert_eq!(runtime.ttl_seconds, Some(3600));
    assert!(runtime.client.is_some());
}

#[test]
fn runtime_resolution_rejects_invalid_ttl() {
    let error = resolve_valkey_analysis_cache_runtime_with_lookup(&|name| match name {
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_URL" => Some("redis://127.0.0.1/".to_string()),
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_TTL_SECS" => Some("invalid".to_string()),
        _ => None,
    })
    .err()
    .unwrap_or_else(|| panic!("invalid ttl should fail"));

    assert!(
        error
            .to_string()
            .contains("XIUXIAN_WENDAO_ANALYZER_VALKEY_TTL_SECS")
    );
}

#[test]
fn runtime_resolution_skips_blank_primary_url_and_blank_ttl() {
    let runtime = resolve_valkey_analysis_cache_runtime_with_lookup(&|name| match name {
        "VALKEY_URL" => Some(" redis://127.0.0.1/2 ".to_string()),
        "XIUXIAN_WENDAO_ANALYZER_VALKEY_URL"
        | "XIUXIAN_WENDAO_ANALYZER_VALKEY_KEY_PREFIX"
        | "XIUXIAN_WENDAO_ANALYZER_VALKEY_TTL_SECS" => Some("   ".to_string()),
        _ => None,
    })
    .unwrap_or_else(|error| panic!("runtime resolution should succeed: {error}"))
    .unwrap_or_else(|| panic!("runtime should exist"));

    assert_eq!(runtime.key_prefix, "xiuxian_wendao:repo_analysis");
    assert_eq!(runtime.ttl_seconds, None);
    assert!(runtime.client.is_some());
}

#[test]
fn runtime_resolution_prefers_toml_values_over_env() {
    let settings = settings_from_yaml(
        r#"
analyzers:
  cache:
    valkey_url: "redis://127.0.0.1:6380/0"
    key_prefix: "xiuxian:test:repo-analysis"
    ttl_seconds: 1800
"#,
    );

    let runtime = resolve_valkey_analysis_cache_runtime_with_settings_and_lookup_for_tests(
        &settings,
        &|_| Some("redis://127.0.0.1:6379/9".to_string()),
    )
    .unwrap_or_else(|error| panic!("runtime resolution should succeed: {error}"))
    .unwrap_or_else(|| panic!("runtime should exist"));

    assert_eq!(runtime.key_prefix, "xiuxian:test:repo-analysis");
    assert_eq!(runtime.ttl_seconds, Some(1800));
    assert!(runtime.client.is_some());
}

#[test]
fn valkey_analysis_key_uses_analysis_identity_even_without_revision() {
    let key = RepositoryAnalysisCacheKey {
        repo_id: "no-revision".to_string().into(),
        checkout_root: "/tmp/no-revision".to_string(),
        analysis_identity: "analysis:no-revision".to_string(),
        checkout_revision: None,
        mirror_revision: None,
        tracking_revision: None,
        plugin_ids: vec!["plugin-a".to_string()],
    };

    assert_eq!(
        valkey_analysis_key(&key, "xiuxian:test"),
        valkey_analysis_key(&key, "xiuxian:test")
    );
    assert!(encode_analysis_payload(&key, &RepositoryAnalysisOutput::default()).is_some());
}

#[test]
fn payload_roundtrip_preserves_analysis_output() {
    let key = sample_cache_key("payload-roundtrip");
    let analysis = RepositoryAnalysisOutput {
        modules: vec![crate::analyzers::ModuleRecord {
            repo_id: key.repo_id.clone().into(),
            module_id: "module:alpha".to_string().into(),
            qualified_name: "Alpha".to_string(),
            path: "src/lib.rs".to_string().into(),
        }],
        ..RepositoryAnalysisOutput::default()
    };
    let payload =
        encode_analysis_payload(&key, &analysis).unwrap_or_else(|| panic!("payload should encode"));
    let decoded = decode_analysis_payload(&key, payload.as_str())
        .unwrap_or_else(|| panic!("payload should decode"));
    let decoded_by_revision = decode_analysis_payload_for_revision(
        key.repo_id.as_str(),
        key.checkout_root.as_str(),
        key.plugin_ids.as_slice(),
        "rev-1",
        payload.as_str(),
    )
    .unwrap_or_else(|| panic!("payload should decode by revision"));

    assert_eq!(decoded, analysis);
    assert_eq!(decoded_by_revision, analysis);
}

#[test]
fn cache_roundtrip_uses_test_shadow_when_no_live_client_is_bound() {
    let cache = ValkeyAnalysisCache::for_tests("xiuxian:test:repo-analysis", Some(60));
    let key = sample_cache_key("shadow-roundtrip");
    let analysis = RepositoryAnalysisOutput {
        modules: vec![crate::analyzers::ModuleRecord {
            repo_id: key.repo_id.clone().into(),
            module_id: "module:alpha".to_string().into(),
            qualified_name: "Alpha".to_string(),
            path: "src/lib.rs".to_string().into(),
        }],
        ..RepositoryAnalysisOutput::default()
    };

    cache.set_analysis(RepositoryAnalysisValkeyScope::current(&key), &analysis);
    let loaded = cache
        .get_analysis(RepositoryAnalysisValkeyScope::current(&key))
        .unwrap_or_else(|| panic!("cached analysis should load"));
    let revision_loaded = cache
        .get_analysis(RepositoryAnalysisValkeyScope::revision(
            key.repo_id.as_str(),
            key.checkout_root.as_str(),
            key.plugin_ids.as_slice(),
            "rev-1",
        ))
        .unwrap_or_else(|| panic!("cached analysis should load by revision"));

    assert_eq!(loaded, analysis);
    assert_eq!(revision_loaded, analysis);
    assert!(
        !valkey_analysis_revision_key(
            key.repo_id.as_str(),
            key.checkout_root.as_str(),
            key.plugin_ids.as_slice(),
            "rev-1",
            "xiuxian:test:repo-analysis",
        )
        .is_empty()
    );
}

#[test]
fn search_query_payload_roundtrip_preserves_value() {
    let key = sample_query_cache_key("query-payload-roundtrip");
    let payload = encode_search_query_payload(&key, &vec!["projected-hit".to_string()])
        .unwrap_or_else(|| panic!("payload should encode"));
    let decoded = decode_search_query_payload::<Vec<String>>(&key, payload.as_str())
        .unwrap_or_else(|| panic!("payload should decode"));

    assert_eq!(decoded, vec!["projected-hit".to_string()]);
    assert!(
        !valkey_search_query_key(&key, "xiuxian:test:repo-analysis").is_empty(),
        "query cache key should exist when analysis identity is stable"
    );
}

#[test]
fn search_query_cache_roundtrip_uses_test_shadow_when_no_live_client_is_bound() {
    let cache = ValkeyAnalysisCache::for_tests("xiuxian:test:repo-analysis", Some(60));
    let key = sample_query_cache_key("query-shadow-roundtrip");
    let value = vec!["projected-hit".to_string()];

    cache.set_search_query(RepositorySearchQueryValkeyScope::new(&key), &value);
    let loaded = cache
        .get_search_query::<Vec<String>>(RepositorySearchQueryValkeyScope::new(&key))
        .unwrap_or_else(|| panic!("cached query result should load"));

    assert_eq!(loaded, value);
}
