use super::{
    ContractOwner, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, GRAPH_STRUCTURAL_FILTER_ROUTE,
    GRAPH_STRUCTURAL_RERANK_ROUTE, GraphStructuralRouteKind, JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION,
    LinkGraphJuliaRerankRuntimeConfig, MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
    MemoryJuliaComputeManifestRow, MemoryJuliaComputeProfile, MemoryJuliaComputeRuntimeConfig,
    PolyglotLane, WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_GNN_REASONING_PROFILE_ID, WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION,
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT,
    WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID, WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    memory_julia_compute_manifest_row_ref, memory_julia_compute_profile_ref,
    memory_julia_compute_profile_refs, wendao_graph_gnn_reasoning_profile_ref,
    wendao_graph_link_evidence_profile_ref, wendao_graph_page_index_reasoning_profile_ref,
    wendaosearch_graph_structural_profile_ref, wendaosearch_legacy_rerank_profile_ref,
};

#[test]
fn profile_ref_projects_runtime_route_and_schema() {
    let mut runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true.into(),
        schema_version: "v2".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    runtime.routes.memory_gate_score = "/memory/custom_gate_score".to_string();

    let reference =
        memory_julia_compute_profile_ref(&runtime, MemoryJuliaComputeProfile::MemoryGateScore);

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, "/memory/custom_gate_score");
    assert_eq!(
        reference.profile.as_deref(),
        Some(MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID)
    );
    assert_eq!(reference.schema_version.as_deref(), Some("v2"));
}

#[test]
fn manifest_row_ref_preserves_julia_owner() {
    let row = MemoryJuliaComputeManifestRow {
        family: "memory".to_string(),
        capability_id: "memory_gate_score".into(),
        profile_id: "memory_gate_score".into(),
        request_schema_id: "memory.gate_score.request.v1".into(),
        response_schema_id: "memory.gate_score.response.v1".into(),
        route: "/memory/gate_score".to_string(),
        health_route: Some("/healthz".into()),
        schema_version: "v1".to_string(),
        timeout_secs: Some(10_u64.into()),
        scenario_pack: None,
        enabled: true.into(),
    };

    let reference = memory_julia_compute_manifest_row_ref(&row);

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, "/memory/gate_score");
    assert_eq!(reference.profile.as_deref(), Some("memory_gate_score"));
    assert_eq!(reference.schema_version.as_deref(), Some("v1"));
}

#[test]
fn profile_refs_cover_staged_memory_profiles() {
    let runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true.into(),
        schema_version: "v1".to_string(),
        ..MemoryJuliaComputeRuntimeConfig::default()
    };

    let references = memory_julia_compute_profile_refs(&runtime);

    assert_eq!(references.len(), MemoryJuliaComputeProfile::ALL.len());
    assert!(
        references
            .iter()
            .all(|reference| reference.owner == ContractOwner::Julia)
    );
}

#[test]
fn wendao_graph_ref_projects_link_evidence_contract() {
    let reference = wendao_graph_link_evidence_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, WENDAO_GRAPH_LINK_EVIDENCE_ROUTE);
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION)
    );
}

#[test]
fn wendao_graph_ref_projects_page_index_reasoning_contract() {
    let reference = wendao_graph_page_index_reasoning_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(
        reference.route,
        WENDAO_GRAPH_PAGE_INDEX_REASONING_HOST_ENTRYPOINT
    );
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_PAGE_INDEX_REASONING_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_EVIDENCE_SCHEMA_VERSION)
    );
}

#[test]
fn wendao_graph_ref_projects_gnn_reasoning_contract() {
    let reference = wendao_graph_gnn_reasoning_profile_ref();

    assert_eq!(reference.lane, PolyglotLane::JuliaCompute);
    assert_eq!(reference.owner, ContractOwner::Julia);
    assert_eq!(reference.route, WENDAO_GRAPH_GNN_REASONING_HOST_ENTRYPOINT);
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAO_GRAPH_GNN_REASONING_PROFILE_ID)
    );
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(WENDAO_GRAPH_GNN_REASONING_SCHEMA_VERSION)
    );
}

#[test]
fn wendaosearch_refs_project_structural_routes() {
    let rerank =
        wendaosearch_graph_structural_profile_ref(GraphStructuralRouteKind::StructuralRerank);
    let filter =
        wendaosearch_graph_structural_profile_ref(GraphStructuralRouteKind::ConstraintFilter);

    assert_eq!(rerank.route, GRAPH_STRUCTURAL_RERANK_ROUTE);
    assert_eq!(
        rerank.profile.as_deref(),
        Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID)
    );
    assert_eq!(
        rerank.schema_version.as_deref(),
        Some(JULIA_GRAPH_STRUCTURAL_SCHEMA_VERSION)
    );
    assert_eq!(filter.route, GRAPH_STRUCTURAL_FILTER_ROUTE);
    assert_eq!(
        filter.profile.as_deref(),
        Some(WENDAOSEARCH_CONSTRAINT_FILTER_PROFILE_ID)
    );
}

#[test]
fn wendaosearch_legacy_ref_projects_runtime_override() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig {
        route: Some("/custom/rerank".into()),
        schema_version: Some("v2".into()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let reference = wendaosearch_legacy_rerank_profile_ref(&runtime);

    assert_eq!(reference.route, "/custom/rerank");
    assert_eq!(
        reference.profile.as_deref(),
        Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
    );
    assert_eq!(reference.schema_version.as_deref(), Some("v2"));
}

#[test]
fn wendaosearch_legacy_ref_uses_default_route_without_runtime_override() {
    let runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let reference = wendaosearch_legacy_rerank_profile_ref(&runtime);

    assert_eq!(reference.route, DEFAULT_JULIA_RERANK_FLIGHT_ROUTE);
    assert_eq!(reference.schema_version.as_deref(), Some("v1"));
}
