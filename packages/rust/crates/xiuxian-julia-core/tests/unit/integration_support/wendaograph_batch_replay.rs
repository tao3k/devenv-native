use super::{
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowOntologyRegistryCacheKey,
};

#[test]
fn ontology_registry_cache_key_tracks_endpoint_and_repo() {
    let config = materialization_config("main").with_timeout_seconds(1);
    let same_semantic_scope = materialization_config("main").with_timeout_seconds(30);
    let different_repo = materialization_config("docs");

    assert_eq!(
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&config),
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&same_semantic_scope)
    );
    assert_ne!(
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&config),
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&different_repo)
    );
}

fn materialization_config(repo: &str) -> SearchStrategyFlowFlightMaterializationConfig {
    SearchStrategyFlowFlightMaterializationConfig::new("http://127.0.0.1:50052", repo)
        .unwrap_or_else(|error| panic!("config should parse: {error}"))
}
