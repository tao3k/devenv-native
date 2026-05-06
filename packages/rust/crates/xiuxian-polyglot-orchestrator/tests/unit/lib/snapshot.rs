use crate::{
    AdmissionBudget, AdmissionDecision, ContractOwner, FallbackEvidence, HealthState, LaneEvidence,
    PolyglotControlSnapshot, PolyglotLane, PressureLevel, ReadinessState, RouteProfileRef,
    SnapshotInvariantError,
};

#[test]
fn snapshot_filters_refs_and_projects_decisions() -> Result<(), SnapshotInvariantError> {
    let python_budget = AdmissionBudget {
        lane: PolyglotLane::PythonDocling,
        max_in_flight: Some(4),
        active_in_flight: 1,
        queue_depth: 0,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Medium,
        fallback_available: true,
    };
    let evidence = LaneEvidence::new(
        PolyglotLane::PythonDocling,
        HealthState::Healthy,
        ReadinessState::Ready,
        PressureLevel::Medium,
        FallbackEvidence::new(true),
    );
    let snapshot = PolyglotControlSnapshot::from_parts(
        vec![
            RouteProfileRef::document_extract("/analysis/document-extract"),
            RouteProfileRef::julia_profile("/memory/gate_score", "memory_gate_score", "v1"),
        ],
        vec![python_budget],
        vec![evidence],
    )?;

    let python_refs = snapshot
        .route_refs_for_lane(PolyglotLane::PythonDocling)
        .collect::<Vec<_>>();
    assert_eq!(python_refs.len(), 1);
    assert_eq!(python_refs[0].owner, ContractOwner::Analyzer);
    assert_eq!(
        snapshot.admission_decision_for_lane(PolyglotLane::PythonDocling),
        Some(AdmissionDecision::Allow {
            lane: PolyglotLane::PythonDocling,
            remaining_permits: 3,
        })
    );
    assert!(
        snapshot
            .evidence_for_lane(PolyglotLane::PythonDocling)
            .is_some()
    );
    Ok(())
}

#[test]
fn snapshot_rejects_duplicate_route_refs() {
    let reference = RouteProfileRef::document_extract("/analysis/document-extract");
    let Err(error) = PolyglotControlSnapshot::from_parts(
        vec![reference.clone(), reference],
        Vec::new(),
        Vec::new(),
    ) else {
        panic!("duplicate route ref should fail");
    };

    assert!(matches!(
        error,
        SnapshotInvariantError::RouteRef {
            lane: PolyglotLane::PythonDocling,
            owner: ContractOwner::Analyzer,
            ..
        }
    ));
}

#[test]
fn snapshot_rejects_duplicate_budget_lanes() {
    let budget = AdmissionBudget::new(PolyglotLane::JuliaCompute);
    let Err(error) =
        PolyglotControlSnapshot::from_parts(Vec::new(), vec![budget, budget], Vec::new())
    else {
        panic!("duplicate budget lane should fail");
    };

    assert_eq!(
        error,
        SnapshotInvariantError::AdmissionBudget {
            lane: PolyglotLane::JuliaCompute,
        }
    );
}

#[test]
fn snapshot_serializes_owned_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = PolyglotControlSnapshot::new().with_route_ref(RouteProfileRef::ocr_shards(
        "/analysis/pdf-ocr-shards",
        "xiuxian_wendao.pdf_ocr_shard_input.v1",
    ))?;

    let serialized = serde_json::to_string(&snapshot)?;

    assert!(serialized.contains("pdf_ocr_shard_input"));
    assert!(serialized.contains("wendao_attachments"));
    Ok(())
}
