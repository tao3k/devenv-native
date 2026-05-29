use crate::wendao::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, DEFAULT_JULIA_SEARCH_LAUNCHER_PATH,
    JULIA_RERANK_CAPABILITY_ID, LinkGraphJuliaRerankRuntimeConfig, build_rerank_provider_binding,
    julia_rerank_provider_selector,
};
use crate::wendao::{JuliaContractMode, JuliaContractPath, JuliaContractRoute, JuliaContractUrl};
use crate::wendao::{
    MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID, MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_FAMILY_ID, MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
    MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID, MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
    MemoryJuliaComputeProfile, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAO_GRAPH_LINK_EVIDENCE_ROUTE, WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
    WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE,
    WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_ROUTE, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE, WendaoGraphAlgorithmComplexity, WendaoGraphAlgorithmId,
    WendaoGraphAlgorithmWorkload, WendaoGraphProfileId, build_memory_julia_compute_binding,
    build_memory_julia_compute_bindings, wendaograph_algorithm_ref, wendaograph_algorithm_refs,
    wendaograph_frontier_algorithm_ref, wendaograph_gnn_algorithm_refs,
    wendaograph_link_graph_algorithm_refs, wendaograph_page_index_algorithm_refs,
    wendaograph_relationship_search_algorithm_refs,
    wendaograph_search_strategy_flow_algorithm_refs,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;

#[test]
fn wendao_profile_ids_are_stable() {
    assert_eq!(
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
        "wendao_graph_link_evidence"
    );
    assert_eq!(
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID,
        "wendao_graph_page_index_reasoning"
    );
    assert_eq!(
        WENDAO_GRAPH_GNN_REASONING_PROFILE_ID,
        "wendao_graph_gnn_reasoning"
    );
    assert_eq!(
        WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
        "wendaosearch_legacy_rerank"
    );
    assert_eq!(
        WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
        "wendaosearch_structural_rerank"
    );
    assert_eq!(
        WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
        "wendaosearch_constraint_filter"
    );
    assert_eq!(WENDAO_GRAPH_LINK_EVIDENCE_ROUTE, "/graph/link/evidence");
    assert_eq!(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, "v0-draft");
    assert_eq!(WENDAOSEARCH_LEGACY_RERANK_ROUTE, "/rerank");
    assert_eq!(
        WENDAOSEARCH_STRUCTURAL_RERANK_ROUTE,
        "/graph/structural/rerank"
    );
    assert_eq!(
        WENDAOSEARCH_CONSTRAINT_FILTER_ROUTE,
        "/graph/structural/filter"
    );
    assert_eq!(WENDAOSEARCH_GRAPH_STRUCTURAL_SCHEMA_VERSION, "v0-draft");
}

#[test]
fn memory_julia_compute_profile_facts_are_stable() {
    assert_eq!(MEMORY_JULIA_COMPUTE_FAMILY_ID, "memory");
    assert_eq!(
        MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
        "episodic_recall"
    );
    assert_eq!(
        MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
        "memory_gate_score"
    );
    assert_eq!(
        MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
        "memory_plan_tuning"
    );
    assert_eq!(
        MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
        "memory_calibration"
    );
    assert_eq!(MemoryJuliaComputeProfile::ALL.len(), 4);

    let profile = MemoryJuliaComputeProfile::parse("memory_gate_score")
        .unwrap_or_else(|| panic!("missing memory_gate_score profile"));
    assert_eq!(
        profile.profile_id(),
        MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID
    );
    assert_eq!(profile.capability_id(), profile.profile_id());
    assert_eq!(
        MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID,
        "memory.gate_score.request.v1"
    );
}

#[test]
fn memory_julia_compute_bindings_materialize_all_profiles() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.plugin_id = "wendao.memory".into();
    runtime.base_url = "http://127.0.0.1:18825".to_string();
    runtime.health_route = Some("/healthz".to_string());
    runtime.max_in_flight_requests = 9;
    runtime.routes.episodic_recall = "/memory/episodic_recall".to_string();
    runtime.routes.memory_gate_score = "/memory/gate_score".to_string();
    runtime.routes.memory_plan_tuning = "/memory/plan_tuning".to_string();
    runtime.routes.memory_calibration = "/memory/calibration".to_string();

    let bindings = build_memory_julia_compute_bindings(&runtime)
        .unwrap_or_else(|error| panic!("bindings should resolve: {error}"));

    assert_eq!(bindings.len(), 4);
    assert_eq!(bindings[0].selector.capability_id.0, "episodic_recall");
    assert_eq!(bindings[1].selector.capability_id.0, "memory_gate_score");
    assert_eq!(bindings[2].selector.capability_id.0, "memory_plan_tuning");
    assert_eq!(bindings[3].selector.capability_id.0, "memory_calibration");
    assert_eq!(bindings[0].selector.provider.0, "wendao.memory");
    assert_eq!(
        bindings[0].endpoint.route.as_deref(),
        Some("/memory/episodic_recall")
    );
    assert_eq!(
        bindings[0].endpoint.health_route.as_deref(),
        Some("/healthz")
    );
    assert_eq!(bindings[0].endpoint.max_in_flight_requests, Some(9));
}

#[test]
fn memory_julia_compute_binding_rejects_invalid_runtime_values() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.plugin_id = "  ".into();
    let Err(error) =
        build_memory_julia_compute_binding(&runtime, MemoryJuliaComputeProfile::EpisodicRecall)
    else {
        panic!("blank provider id should fail");
    };
    assert!(error.to_string().contains("plugin_id"));

    runtime.plugin_id = "wendao.memory".into();
    runtime.health_route = Some("/".to_string());
    let Err(error) =
        build_memory_julia_compute_binding(&runtime, MemoryJuliaComputeProfile::EpisodicRecall)
    else {
        panic!("invalid health_route should fail");
    };
    assert!(error.to_string().contains("health_route"));
}

#[test]
fn link_graph_rerank_provider_binding_uses_runtime_route_and_launch_config() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        base_url: Some(JuliaContractUrl::from("http://127.0.0.1:18080")),
        route: Some(JuliaContractRoute::from("/custom-rerank")),
        service_mode: Some(JuliaContractMode::from("stream")),
        search_config_path: Some(JuliaContractPath::from("config/live/solver_demo.toml")),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let binding = build_rerank_provider_binding(&runtime);

    assert_eq!(binding.selector, julia_rerank_provider_selector());
    assert_eq!(binding.selector.capability_id.0, JULIA_RERANK_CAPABILITY_ID);
    assert_eq!(binding.endpoint.route.as_deref(), Some("/custom-rerank"));
    assert_eq!(
        binding
            .launch
            .unwrap_or_else(|| panic!("launch config missing"))
            .launcher_path,
        DEFAULT_JULIA_SEARCH_LAUNCHER_PATH
    );
}

#[test]
fn link_graph_rerank_provider_binding_defaults_route_when_not_configured() {
    let binding = build_rerank_provider_binding(&LinkGraphJuliaRerankRuntimeConfig::default());

    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some(DEFAULT_JULIA_RERANK_FLIGHT_ROUTE)
    );
}

#[test]
fn wendaograph_identity_newtypes_are_transparent() {
    assert_eq!(
        WendaoGraphAlgorithmId("link_graph.components").0,
        "link_graph.components"
    );
    assert_eq!(
        WendaoGraphProfileId(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID).0,
        WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID
    );
}

#[test]
fn wendaograph_workload_builder_preserves_owner_facts() {
    let workload = WendaoGraphAlgorithmWorkload::new()
        .with_rows(21)
        .with_graph_size(34, 55)
        .with_feature_columns(8)
        .with_byte_size(144);

    assert_eq!(workload.rows, 21);
    assert_eq!(workload.nodes, 34);
    assert_eq!(workload.edges, 55);
    assert_eq!(workload.feature_columns, 8);
    assert_eq!(workload.byte_size, 144);
}

#[test]
fn wendaograph_catalog_covers_polyglot_owned_algorithm_families() {
    let references = wendaograph_algorithm_refs();

    assert_eq!(wendaograph_link_graph_algorithm_refs().len(), 17);
    assert_eq!(wendaograph_relationship_search_algorithm_refs().len(), 10);
    assert_eq!(wendaograph_page_index_algorithm_refs().len(), 3);
    assert_eq!(wendaograph_search_strategy_flow_algorithm_refs().len(), 4);
    assert_eq!(wendaograph_gnn_algorithm_refs().len(), 4);
    assert_eq!(references.len(), 38);
    assert!(
        references
            .iter()
            .any(|reference| reference.algorithm_id == "relationship_search.graph_search_ranking")
    );
}

#[test]
fn wendaograph_catalog_preserves_profile_and_complexity_facts() {
    let topology = wendaograph_algorithm_ref(WendaoGraphAlgorithmId("link_graph.topology_core"))
        .unwrap_or_else(|| panic!("missing link_graph.topology_core"));
    let disclosure =
        wendaograph_algorithm_ref(WendaoGraphAlgorithmId("page_index.disclosure_trace"))
            .unwrap_or_else(|| panic!("missing page_index.disclosure_trace"));
    let frontier = wendaograph_frontier_algorithm_ref("page_index_seed")
        .unwrap_or_else(|| panic!("missing page_index_seed frontier algorithm"));

    assert_eq!(topology.profile_id, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID);
    assert_eq!(topology.complexity, WendaoGraphAlgorithmComplexity::Heavy);
    assert!(topology.is_heavy());
    assert_eq!(
        disclosure.complexity,
        WendaoGraphAlgorithmComplexity::Simple
    );
    assert_eq!(frontier.algorithm_id, "page_index.reasoning_frontier");
    assert_eq!(
        frontier.profile_id,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID
    );
}
