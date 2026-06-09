use std::fs;

use super::{
    WENDAOGRAPH_JULIA_PROJECT_ENV, WENDAOGRAPH_PACKAGE_DIR_ENV,
    local_wendaograph_package_available,
    run_wendaograph_search_strategy_flow_json_with_candidate_batch,
    search_strategy_flow_registry_authority_candidate_input_batch,
};

#[test]
fn search_strategy_flow_registry_authority_batch_replays_through_julia()
-> Result<(), Box<dyn std::error::Error>> {
    if !local_wendaograph_package_available() {
        eprintln!(
            "skipping WendaoGraph registry authority replay; set {WENDAOGRAPH_PACKAGE_DIR_ENV} or {WENDAOGRAPH_JULIA_PROJECT_ENV}"
        );
        return Ok(());
    }

    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    fs::write(
        root.join("wendao.toml"),
        r#"
[link_graph.projects.main]
root = "."
dirs = ["docs"]

[link_graph.projects.wendaograph]
url = "https://example.invalid/wendaograph.git"
plugins = ["julia"]
"#,
    )?;

    let batch = search_strategy_flow_registry_authority_candidate_input_batch(root)?;
    let trace = run_wendaograph_search_strategy_flow_json_with_candidate_batch(
        "ownership boundary registry authority",
        root,
        batch,
    )?;
    let trace: serde_json::Value = serde_json::from_str(&trace)?;
    let candidates = trace
        .get("candidates")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("candidates must be an array"));
    let frontier = trace
        .get("frontier")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("frontier must be an array"));
    let routes = trace
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    let discovery_contract = trace
        .get("candidateDiscoveryContract")
        .unwrap_or_else(|| panic!("candidateDiscoveryContract must be present"));

    assert_eq!(candidates.len(), 2);
    assert!(
        frontier.iter().any(|candidate| {
            candidate
                .get("candidateId")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|candidate_id| candidate_id.contains("registry-authority"))
        }),
        "registry authority candidates must reach the Julia frontier"
    );
    assert_eq!(
        discovery_contract.get("structuredSurfaceId"),
        Some(&serde_json::json!("registry-authority"))
    );
    assert_eq!(
        discovery_contract
            .get("discoveryReceipt")
            .and_then(|receipt| receipt.get("transport")),
        Some(&serde_json::json!("rust-config-scan"))
    );
    assert!(
        routes.iter().any(|route| {
            route.get("sourcePath").and_then(serde_json::Value::as_str) == Some("wendao.toml")
        }),
        "registry authority replay must plan routes against root wendao.toml"
    );
    Ok(())
}
