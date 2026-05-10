use super::wendao_polyglot_control_snapshot_from_parts;
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, ContractOwner, PolyglotLane, PressureLevel, ReadinessState, RejectionReason,
    SnapshotInvariantError,
};
use xiuxian_wendao_julia::compatibility::link_graph::LinkGraphJuliaRerankRuntimeConfig;
use xiuxian_wendao_julia::memory::MemoryJuliaComputeProfile;
use xiuxian_wendao_julia::polyglot::{
    WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID, WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID,
    WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
};
use xiuxian_wendao_runtime::config::MemoryJuliaComputeRuntimeConfig;
use xiuxian_wendao_runtime::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

#[test]
fn host_snapshot_combines_document_memory_and_graph_refs() -> Result<(), SnapshotInvariantError> {
    let memory_runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: true,
        schema_version: "v1".to_string(),
        max_in_flight_requests: 6,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let link_graph_julia_runtime = LinkGraphJuliaRerankRuntimeConfig {
        route: Some("/custom/rerank".to_string().into()),
        schema_version: Some("v2".to_string().into()),
        ..LinkGraphJuliaRerankRuntimeConfig::default()
    };

    let snapshot = wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        &link_graph_julia_runtime,
        2,
        1,
        ReadinessState::Ready,
        PressureLevel::Medium,
    )?;

    assert_eq!(
        snapshot.route_refs().len(),
        1 + MemoryJuliaComputeProfile::ALL.len() + 4
    );
    assert_eq!(
        snapshot
            .route_refs_for_owner(ContractOwner::Analyzer)
            .map(|reference| reference.route.as_str())
            .collect::<Vec<_>>(),
        vec![ANALYSIS_DOCUMENT_EXTRACT_ROUTE]
    );
    assert_eq!(
        snapshot
            .route_refs_for_owner(ContractOwner::Julia)
            .filter(|reference| reference.lane == PolyglotLane::JuliaCompute)
            .count(),
        MemoryJuliaComputeProfile::ALL.len() + 4
    );
    assert!(snapshot.route_refs().iter().any(
        |reference| reference.profile.as_deref() == Some(WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID)
    ));
    assert!(
        snapshot
            .route_refs()
            .iter()
            .any(|reference| reference.profile.as_deref()
                == Some(WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID))
    );
    assert!(snapshot.route_refs().iter().any(|reference| {
        reference.profile.as_deref() == Some(WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID)
            && reference.route == "/custom/rerank"
            && reference.schema_version.as_deref() == Some("v2")
    }));
    assert_eq!(
        snapshot.admission_decision_for_lane(PolyglotLane::JuliaCompute),
        Some(AdmissionDecision::Allow {
            lane: PolyglotLane::JuliaCompute,
            remaining_permits: 4,
        })
    );
    assert!(snapshot.lane_evidence().is_empty());
    Ok(())
}

#[test]
fn host_snapshot_uses_runtime_admission_fallback_state() -> Result<(), SnapshotInvariantError> {
    let memory_runtime = MemoryJuliaComputeRuntimeConfig {
        enabled: false,
        max_in_flight_requests: 0,
        ..MemoryJuliaComputeRuntimeConfig::default()
    };
    let link_graph_julia_runtime = LinkGraphJuliaRerankRuntimeConfig::default();

    let snapshot = wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        &link_graph_julia_runtime,
        0,
        0,
        ReadinessState::Disabled,
        PressureLevel::Low,
    )?;

    let Some(budget) = snapshot.admission_budget_for_lane(PolyglotLane::JuliaCompute) else {
        panic!("Julia admission budget");
    };
    assert!(budget.fallback_available);
    assert_eq!(
        budget.decide(),
        AdmissionDecision::Reject {
            lane: PolyglotLane::JuliaCompute,
            reason: RejectionReason::LaneDisabled,
        }
    );
    Ok(())
}
