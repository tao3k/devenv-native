use super::{
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowOntologyRegistryCacheKey,
};

#[test]
fn ontology_registry_cache_key_tracks_endpoint_and_repo() {
    let config =
        SearchStrategyFlowFlightMaterializationConfig::new("http://127.0.0.1:50052", "main")
            .expect("config should parse")
            .with_timeout_seconds(1);
    let same_semantic_scope =
        SearchStrategyFlowFlightMaterializationConfig::new("http://127.0.0.1:50052", "main")
            .expect("config should parse")
            .with_timeout_seconds(30);
    let different_repo =
        SearchStrategyFlowFlightMaterializationConfig::new("http://127.0.0.1:50052", "docs")
            .expect("config should parse");

    assert_eq!(
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&config),
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&same_semantic_scope)
    );
    assert_ne!(
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&config),
        SearchStrategyFlowOntologyRegistryCacheKey::from_config(&different_repo)
    );
}
