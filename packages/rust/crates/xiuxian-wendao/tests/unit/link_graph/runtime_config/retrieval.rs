use crate::link_graph::runtime_config::models::LinkGraphSemanticIgnitionBackend;
use crate::link_graph::runtime_config::resolve_link_graph_retrieval_policy_runtime;
use crate::link_graph::set_link_graph_wendao_config_override;
use serial_test::serial;
use std::fs;
use xiuxian_wendao_runtime::transport::CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER;

#[test]
#[serial]
fn test_retrieval_runtime_resolves_semantic_ignition_config()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    let shared_path = temp.path().join("wendao.shared.toml");
    fs::write(
        &shared_path,
        r#"[semantic_ignition]
backend = "openai-compatible"
vector_store_path = ".cache/glm-anchor-store"
table_name = "glm_anchor_index"
embedding_base_url = "http://127.0.0.1:11434"
embedding_model = "glm-5"
"#,
    )?;
    fs::write(
        &config_path,
        r#"[link_graph.retrieval]
imports = ["wendao.shared.toml"]
mode = "hybrid"
candidate_multiplier = 3
max_sources = 5
graph_rows_per_source = 4
"#,
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let runtime = resolve_link_graph_retrieval_policy_runtime();
    assert_eq!(
        runtime.semantic_ignition.backend,
        LinkGraphSemanticIgnitionBackend::OpenAiCompatible
    );
    assert_eq!(runtime.candidate_multiplier, 3);
    assert_eq!(runtime.max_sources, 5);
    assert_eq!(runtime.graph_rows_per_source, 4);
    assert_eq!(
        runtime.semantic_ignition.vector_store_path.as_deref(),
        Some(".cache/glm-anchor-store")
    );
    assert_eq!(
        runtime.semantic_ignition.table_name.as_deref(),
        Some("glm_anchor_index")
    );
    assert_eq!(
        runtime.semantic_ignition.embedding_base_url.as_deref(),
        Some("http://127.0.0.1:11434")
    );
    assert_eq!(
        runtime.semantic_ignition.embedding_model.as_deref(),
        Some("glm-5")
    );
    assert!(runtime.rerank_binding().is_none());
    assert!(runtime.rerank_schema_version().is_none());
    assert!(runtime.rerank_score_weights().is_none());

    Ok(())
}

#[test]
fn canonical_transport_preference_order_is_flight_first() {
    assert_eq!(
        CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER,
        [xiuxian_wendao_core::transport::PluginTransportKind::ArrowFlight]
    );
}
