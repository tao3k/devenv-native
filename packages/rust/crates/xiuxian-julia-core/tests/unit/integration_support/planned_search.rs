use super::{
    julia_planned_search_openai_runtime_config_toml,
    julia_planned_search_vector_store_runtime_config_toml,
};

#[test]
fn planned_search_openai_runtime_config_keeps_stable_shape() {
    let rendered = julia_planned_search_openai_runtime_config_toml(
        "/tmp/vector-store",
        "http://127.0.0.1:9999",
        "http://127.0.0.1:8088",
    );

    assert!(rendered.contains("backend = \"openai-compatible\""));
    assert!(rendered.contains("embedding_model = \"glm-5\""));
    assert!(rendered.contains("route = \"/rerank\""));
    assert!(rendered.contains("timeout_secs = 30"));
}

#[test]
fn planned_search_vector_store_runtime_config_keeps_stable_shape() {
    let rendered = julia_planned_search_vector_store_runtime_config_toml(
        "/tmp/vector-store",
        "http://127.0.0.1:8088",
    );

    assert!(rendered.contains("backend = \"vector-store\""));
    assert!(!rendered.contains("embedding_base_url"));
    assert!(rendered.contains("route = \"/rerank\""));
    assert!(rendered.contains("schema_version = \"v1\""));
}
