use crate::memory::host::gate_score::{
    MemoryGateScoreEvidenceRow, build_memory_gate_score_request_batch_from_evidence,
    build_memory_gate_score_request_rows_from_evidence,
};
use crate::memory::host::{MemoryLifecycleState, MemoryUtilityLedger};

fn sample_evidence_row(
    memory_id: &str,
    scenario_pack: Option<String>,
    current_state: MemoryLifecycleState,
) -> MemoryGateScoreEvidenceRow {
    MemoryGateScoreEvidenceRow {
        memory_id: memory_id.to_string(),
        scenario_pack,
        ledger: MemoryUtilityLedger {
            react_revalidation_score: 0.91,
            graph_consistency_score: 0.88,
            omega_alignment_score: 0.93,
            ttl_score: 0.66,
            utility_score: 0.82,
            q_value: 0.84,
            usage_count: 6,
            failure_rate: 1.0 / 6.0,
        },
        current_state,
    }
}

fn sample_cooling_evidence_row(memory_id: &str) -> MemoryGateScoreEvidenceRow {
    MemoryGateScoreEvidenceRow {
        memory_id: memory_id.to_string(),
        scenario_pack: None,
        ledger: MemoryUtilityLedger {
            react_revalidation_score: 0.77,
            graph_consistency_score: 0.74,
            omega_alignment_score: 0.81,
            ttl_score: 0.58,
            utility_score: 0.7,
            q_value: 0.62,
            usage_count: 2,
            failure_rate: 0.5,
        },
        current_state: MemoryLifecycleState::Cooling,
    }
}

#[test]
fn build_memory_gate_score_request_rows_from_evidence_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = sample_evidence_row(
        "memory-alpha",
        Some("searchinfra".to_string()),
        MemoryLifecycleState::Active,
    );

    let rows = build_memory_gate_score_request_rows_from_evidence(&[evidence])?;

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.memory_id, "memory-alpha");
    assert_eq!(row.scenario_pack.as_deref(), Some("searchinfra"));
    assert!((row.react_revalidation_score - 0.91).abs() < 1e-6);
    assert!((row.graph_consistency_score - 0.88).abs() < 1e-6);
    assert!((row.omega_alignment_score - 0.93).abs() < 1e-6);
    assert!((row.q_value - 0.84).abs() < 1e-6);
    assert_eq!(row.usage_count, 6);
    assert!((row.failure_rate - (1.0 / 6.0)).abs() < 1e-6);
    assert!((row.ttl_score - 0.66).abs() < 1e-6);
    assert_eq!(row.current_state, "active");

    Ok(())
}

#[test]
fn build_memory_gate_score_request_batch_from_evidence_materializes_staged_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence_rows = vec![
        sample_evidence_row(
            "memory-alpha",
            Some("searchinfra".to_string()),
            MemoryLifecycleState::Active,
        ),
        sample_cooling_evidence_row("memory-beta"),
    ];

    let batch = build_memory_gate_score_request_batch_from_evidence(&evidence_rows)?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().fields().len(), 10);
    assert!(batch.column_by_name("memory_id").is_some());
    assert!(batch.column_by_name("ttl_score").is_some());
    assert!(batch.column_by_name("current_state").is_some());

    Ok(())
}

#[test]
fn build_memory_gate_score_request_batch_from_evidence_rejects_invalid_memory_id() {
    let evidence_rows = vec![MemoryGateScoreEvidenceRow {
        memory_id: "   ".to_string(),
        scenario_pack: None,
        ledger: MemoryUtilityLedger {
            react_revalidation_score: 0.9,
            graph_consistency_score: 0.8,
            omega_alignment_score: 0.7,
            ttl_score: 0.6,
            utility_score: 0.75,
            q_value: 0.85,
            usage_count: 4,
            failure_rate: 0.2,
        },
        current_state: MemoryLifecycleState::RevalidatePending,
    }];

    let Err(error) = build_memory_gate_score_request_batch_from_evidence(&evidence_rows) else {
        panic!("blank memory_id must fail");
    };

    assert!(error.to_string().contains("memory_id"));
}
