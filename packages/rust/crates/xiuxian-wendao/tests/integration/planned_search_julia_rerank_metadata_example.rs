#![cfg(feature = "julia")]

//! Integration test for planned-search Julia rerank against the official
//! `WendaoArrow` stream metadata example.

use serial_test::serial;
use std::fs;
use xiuxian_db_store::VectorStore;
use xiuxian_wendao::{
    LinkGraphIndex, LinkGraphSearchOptions, set_link_graph_wendao_config_override,
};
use xiuxian_wendao_builtin::{
    linked_builtin_julia_planned_search_vector_store_runtime_config_toml as julia_planned_search_vector_store_runtime_config_toml,
    linked_builtin_spawn_wendaoarrow_stream_metadata_service as spawn_wendaoarrow_stream_metadata_service,
};

#[test]
#[serial(link_graph_runtime_config)]
fn test_planned_search_payload_applies_official_stream_metadata_example()
-> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let temp = tempfile::tempdir()?;
    fs::write(
        temp.path().join("alpha.md"),
        "# Alpha\n\nalpha signal remains dominant.\n",
    )?;
    fs::write(
        temp.path().join("beta.md"),
        "# Beta\n\nbeta remains as the contrasting note.\n",
    )?;

    let vector_store_path = temp.path().join("vector-store");
    let store = runtime.block_on(VectorStore::new(
        vector_store_path.to_string_lossy().as_ref(),
        Some(3),
    ))?;
    runtime.block_on(store.add_documents(
        "wendao_semantic_docs",
        vec!["alpha".to_string(), "beta".to_string()],
        vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]],
        vec!["alpha anchor".to_string(), "beta anchor".to_string()],
        vec!["{}".to_string(), "{}".to_string()],
    ))?;

    let (server_base_url, mut server_guard) =
        runtime.block_on(spawn_wendaoarrow_stream_metadata_service());
    let config_path = temp.path().join("wendao.toml");
    fs::write(
        &config_path,
        julia_planned_search_vector_store_runtime_config_toml(
            vector_store_path.to_string_lossy().as_ref(),
            server_base_url.as_str(),
        ),
    )?;
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let index = LinkGraphIndex::build(temp.path())?;
    let payload = index.search_planned_payload_with_agentic_query_vector(
        "alpha signal",
        &[1.0, 0.0, 0.0],
        2,
        LinkGraphSearchOptions::default(),
        None,
        None,
    );
    server_guard.kill();

    assert_eq!(
        payload
            .julia_rerank
            .as_ref()
            .map(|telemetry| telemetry.applied),
        Some(true),
        "unexpected Julia rerank telemetry: {:?}",
        payload.julia_rerank
    );
    assert_eq!(
        payload
            .julia_rerank
            .as_ref()
            .map(|telemetry| telemetry.response_row_count),
        Some(2)
    );
    assert_eq!(
        payload
            .julia_rerank
            .as_ref()
            .map(|telemetry| telemetry.trace_ids.clone()),
        Some(vec![
            "plugin-rerank:xiuxian-wendao-julia:alpha_signal".to_string()
        ])
    );
    assert!(
        payload
            .julia_rerank
            .as_ref()
            .and_then(|telemetry| telemetry.error.as_ref())
            .is_none()
    );
    assert_eq!(payload.quantum_contexts.len(), 2);

    Ok(())
}
