use super::{
    LINK_GRAPH_LEGACY_RERANK_ROUTE_ENV, LINK_GRAPH_LEGACY_RERANK_ROUTE_KEY,
    resolve_optional_setting_or_env_with_lookup, wendao_polyglot_control_snapshot_from_parts,
};
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, ContractOwner, MemoryJuliaComputeProfile, PolyglotLane, PressureLevel,
    ReadinessState, RejectionReason, SnapshotInvariantError, WENDAO_GRAPH_LINK_EVIDENCE_PROFILE_ID,
    WENDAOSEARCH_LEGACY_RERANK_PROFILE_ID, WENDAOSEARCH_STRUCTURAL_RERANK_PROFILE_ID,
    WendaoSearchLegacyRerankProfileRefInput,
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
    let legacy_rerank = WendaoSearchLegacyRerankProfileRefInput {
        route: Some("/custom/rerank"),
        schema_version: Some("v2"),
    };

    let snapshot = wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        legacy_rerank,
        2,
        1,
        ReadinessState::Ready,
        PressureLevel::Medium,
    )?;

    assert_eq!(
        snapshot.route_refs().len(),
        1 + MemoryJuliaComputeProfile::ALL.len() + 6
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
        MemoryJuliaComputeProfile::ALL.len() + 6
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
    let snapshot = wendao_polyglot_control_snapshot_from_parts(
        &memory_runtime,
        WendaoSearchLegacyRerankProfileRefInput::default(),
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

#[test]
fn legacy_rerank_ref_resolution_prefers_settings_before_env() {
    let settings = match serde_yaml::from_str::<serde_yaml::Value>(
        r#"
link_graph:
  retrieval:
    julia_rerank:
      route: " /settings/rerank "
"#,
    ) {
        Ok(settings) => settings,
        Err(error) => panic!("valid settings fixture: {error}"),
    };

    let route = resolve_optional_setting_or_env_with_lookup(
        &settings,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_KEY,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_ENV,
        |_| Some("/env/rerank".to_string()),
    );

    assert_eq!(route.as_deref(), Some("/settings/rerank"));
}

#[test]
fn legacy_rerank_ref_resolution_falls_back_to_env_when_setting_is_blank() {
    let settings = match serde_yaml::from_str::<serde_yaml::Value>(
        r#"
link_graph:
  retrieval:
    julia_rerank:
      route: "   "
"#,
    ) {
        Ok(settings) => settings,
        Err(error) => panic!("valid settings fixture: {error}"),
    };

    let route = resolve_optional_setting_or_env_with_lookup(
        &settings,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_KEY,
        LINK_GRAPH_LEGACY_RERANK_ROUTE_ENV,
        |_| Some(" /env/rerank ".to_string()),
    );

    assert_eq!(route.as_deref(), Some("/env/rerank"));
}
