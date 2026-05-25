use std::{env, fs, path::PathBuf};

use super::{
    REGISTRY_METADATA_CANDIDATE_SOURCE, audit_search_strategy_flow_registry_authority,
    search_strategy_flow_registry_authority_candidate_input_batch,
};

#[test]
fn registry_authority_audit_follows_root_wendao_toml_imports()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    fs::create_dir_all(root.join("assets/wendao"))?;
    fs::write(
        root.join("wendao.toml"),
        r#"
imports = ["assets/wendao/imported.toml"]

[link_graph.projects.local]
root = "."
dirs = ["docs"]
plugins = ["markdown"]

[link_graph.projects.remote]
url = "https://example.invalid/root.git"
refresh = "on-demand"
plugins = ["julia"]
"#,
    )?;
    fs::write(
        root.join("assets/wendao/imported.toml"),
        r#"
[link_graph.projects.remote]
url = "https://example.invalid/imported.git"
plugins = ["python"]

[link_graph.projects."remote-lib"]
url = "https://example.invalid/remote-lib.git"
plugins = ["modelica"]
"#,
    )?;

    let audit = audit_search_strategy_flow_registry_authority(root)?;
    assert_eq!(audit.config_surface, "root-wendao.toml");
    assert_eq!(audit.configured_project_count, 3);
    assert_eq!(audit.local_project_count, 1);
    assert_eq!(audit.remote_project_count, 2);
    assert_eq!(audit.visited_config_count, 2);
    assert_eq!(
        audit
            .rows
            .iter()
            .find(|project| project.project_id == "remote")
            .and_then(|project| project.url.as_deref()),
        Some("https://example.invalid/root.git")
    );
    Ok(())
}

#[test]
fn registry_authority_batch_uses_authority_route_markers_and_receipt()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    fs::write(
        root.join("wendao.toml"),
        r#"
[link_graph.projects.main]
root = "."
dirs = ["docs"]

[link_graph.projects."GraphSignals.jl"]
url = "https://example.invalid/GraphSignals.jl.git"
plugins = ["julia"]
"#,
    )?;

    let batch = search_strategy_flow_registry_authority_candidate_input_batch(root)?;
    assert_eq!(batch.source, REGISTRY_METADATA_CANDIDATE_SOURCE);
    assert_eq!(batch.row_count, 2);
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("wendao.toml|registry-authority-source-authority-package-owner-main")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("registry-authority-source-authority-package-owner-graphsignals-jl")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("source-authority")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("package-owner")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("plugin:julia")
    );

    let receipt: serde_json::Value = serde_json::from_str(&batch.discovery_receipt_json)?;
    assert_eq!(
        receipt.get("candidateInputSource"),
        Some(&serde_json::json!(REGISTRY_METADATA_CANDIDATE_SOURCE))
    );
    assert_eq!(
        receipt.get("transport"),
        Some(&serde_json::json!("rust-config-scan"))
    );
    assert_eq!(
        receipt.get("configuredProjectCount"),
        Some(&serde_json::json!(2))
    );
    Ok(())
}

#[test]
fn registry_authority_batch_covers_real_root_wendao_toml_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let root = repository_root();
    let audit = audit_search_strategy_flow_registry_authority(root.as_path())?;
    let batch = search_strategy_flow_registry_authority_candidate_input_batch(root.as_path())?;

    assert_eq!(audit.config_surface, "root-wendao.toml");
    assert_eq!(audit.configured_project_count, 181);
    assert_eq!(batch.source, REGISTRY_METADATA_CANDIDATE_SOURCE);
    assert_eq!(batch.row_count, 181);
    assert_eq!(
        batch
            .candidate_input_arrow_snapshot()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        181
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("registry-authority-source-authority-package-owner-main")
    );
    assert!(
        batch
            .candidate_input_arrow_snapshot()
            .contains("registry-authority-source-authority-package-owner-lance")
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
