use std::{env, path::PathBuf};

use super::{
    CODE_INTELLIGENCE_CANDIDATE_SOURCE, audit_search_strategy_flow_code_intelligence_inventory,
    search_strategy_flow_code_intelligence_inventory_candidate_input_batch,
};

#[test]
fn code_intelligence_inventory_covers_real_git_tracked_configured_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let root = repository_root();
    let audit = audit_search_strategy_flow_code_intelligence_inventory(root.as_path())?;

    assert!(
        audit
            .configured_include_dirs
            .contains(&"packages/rust/crates/xiuxian-wendao".to_owned())
    );
    assert!(
        audit
            .configured_include_dirs
            .contains(&"packages/rust/crates/xiuxian-wendao/src/link_graph".to_owned())
    );
    assert!(
        audit
            .configured_include_dirs
            .contains(&"packages/python/wendao-knowledge-retrieval-benchmark".to_owned())
    );
    assert!(audit.rust_control_plane_count > 0);
    assert!(audit.link_graph_source_count > 0);
    assert!(audit.toml_config_count > 0);
    assert!(audit.benchmark_python_count > 0);
    assert!(audit.rust_control_plane_count >= audit.link_graph_source_count);
    assert_eq!(
        audit.total_candidate_count,
        audit.rust_control_plane_count
            + audit.link_graph_source_count
            + audit.toml_config_count
            + audit.benchmark_python_count
    );
    Ok(())
}

#[test]
fn code_intelligence_inventory_batch_uses_surface_source_not_transport_name()
-> Result<(), Box<dyn std::error::Error>> {
    let root = repository_root();
    let batch =
        search_strategy_flow_code_intelligence_inventory_candidate_input_batch(root.as_path())?;
    let audit = audit_search_strategy_flow_code_intelligence_inventory(root.as_path())?;

    assert_eq!(batch.source, CODE_INTELLIGENCE_CANDIDATE_SOURCE);
    assert_eq!(batch.row_count, audit.total_candidate_count);
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("rust-control-plane-source-")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("link-graph-source-focus-")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("toml-config-boundary-")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("benchmark-python-adapter-")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("git-tracked-inventory")
    );

    let receipt: serde_json::Value = serde_json::from_str(&batch.discovery_receipt_json)?;
    assert_eq!(
        receipt.get("candidateInputSource"),
        Some(&serde_json::json!(CODE_INTELLIGENCE_CANDIDATE_SOURCE))
    );
    assert_eq!(
        receipt.get("transport"),
        Some(&serde_json::json!("git-ls-files"))
    );
    assert_eq!(
        receipt.get("mergedCandidateCount"),
        Some(&serde_json::json!(audit.total_candidate_count))
    );
    Ok(())
}

fn repository_root() -> PathBuf {
    env::var_os("PRJ_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(4)
                .unwrap_or_else(|| panic!("resolve repository root from Cargo manifest"))
                .to_path_buf()
        },
        PathBuf::from,
    )
}
