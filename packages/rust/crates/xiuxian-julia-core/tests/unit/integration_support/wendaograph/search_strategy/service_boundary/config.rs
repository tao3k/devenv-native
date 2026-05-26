use std::fs;

use super::support::{
    assert_live_bridge_source_is_gateway_config_free, crate_root, display_relative,
    read_crate_source,
};

#[test]
fn search_strategy_flow_rust_bridge_keeps_gateway_config_out_of_live_client_boundary() {
    for relative_path in [
        "src/integration_support/search_strategy_flow_flight/candidate_source/mod.rs",
        "src/integration_support/search_strategy_flow_flight/admission.rs",
        "src/integration_support/search_strategy_flow_flight/client.rs",
        "src/integration_support/search_strategy_flow_flight/config.rs",
        "src/integration_support/search_strategy_flow_flight/materialization.rs",
        "src/integration_support/search_strategy_flow_flight/metadata.rs",
        "src/integration_support/search_strategy_flow_flight/query.rs",
        "src/integration_support/search_strategy_flow_flight/rows.rs",
        "src/integration_support/wendaograph/search_strategy_routes.rs",
    ] {
        assert_live_bridge_source_is_gateway_config_free(relative_path);
    }
}

#[test]
fn search_strategy_flow_rust_bridge_documents_studio_backed_client_role() {
    let flight_mod =
        read_crate_source("src/integration_support/search_strategy_flow_flight/mod.rs");
    let flight_config =
        read_crate_source("src/integration_support/search_strategy_flow_flight/config.rs");
    let routes = read_crate_source("src/integration_support/wendaograph/search_strategy_routes.rs");

    assert!(flight_mod.contains("Studio-backed `SearchStrategyFlow` routes"));
    assert!(flight_config.contains("Studio-backed `SearchStrategyFlow` Flight"));
    assert!(flight_config.contains("Base URL of the Studio Arrow Flight endpoint"));
    assert!(routes.contains("\"gateway-flight-trace\""));
}

#[test]
fn search_strategy_flow_gateway_config_reads_stay_in_offline_audit_only() {
    let live_flight_dir = crate_root().join("src/integration_support/search_strategy_flow_flight");
    let live_sources = fs::read_dir(&live_flight_dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", live_flight_dir.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("read live bridge source entry: {error}"))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect::<Vec<_>>();

    for source_path in live_sources {
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", source_path.display()));
        assert!(
            !source.contains("OFFLINE_AUDIT_ROOT_WENDAO_CONFIG_PATH"),
            "{} must not depend on offline audit config reads",
            display_relative(&source_path)
        );
    }

    let offline_audit_source = read_crate_source(
        "src/integration_support/search_strategy_flow_candidates/code_inventory.rs",
    );
    assert!(offline_audit_source.contains("OFFLINE_AUDIT_ROOT_WENDAO_CONFIG_PATH"));
}

#[test]
fn search_strategy_flow_rust_bridge_does_not_introduce_julia_embedding_runtime() {
    let cargo_toml = read_crate_source("Cargo.toml");
    assert!(
        !cargo_toml.contains("jlrs"),
        "SearchStrategyFlow Rust bridge must not add jlrs/Rust-embedded Julia runtime"
    );
}
