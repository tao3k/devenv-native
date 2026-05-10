use tempfile::TempDir;
use xiuxian_memory_engine::{
    Episode, EpisodeStore, MemoryLifecycleState, MemoryUtilityLedger, StoreConfig,
};

use crate::memory::host::gate_score::{
    MemoryGateScoreEvidenceRow, MemoryGateScoreEvidenceSignals, MemoryGateScoreStoreEvidenceInput,
    build_memory_gate_score_evidence_row_from_episode,
    build_memory_gate_score_evidence_row_from_store,
    build_memory_gate_score_request_batch_from_evidence,
    build_memory_gate_score_request_rows_from_evidence,
};

fn make_store() -> Result<(TempDir, EpisodeStore), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let store = EpisodeStore::new(StoreConfig {
        path: temp.path().to_string_lossy().to_string(),
        embedding_dim: 3,
        table_name: "gate-score-host-staging".to_string(),
    });
    Ok((temp, store))
}

fn sample_episode(memory_id: &str) -> Episode {
    let mut episode = Episode::new_scoped(
        memory_id.to_string(),
        "intent".to_string(),
        vec![0.1, 0.2, 0.3],
        "experience".to_string(),
        "completed".to_string(),
        "alpha",
    );
    episode.q_value = 0.84;
    episode.success_count = 5;
    episode.failure_count = 1;
    episode.retrieval_count = 6;
    episode
}

fn gate_score_signals(current_state: MemoryLifecycleState) -> MemoryGateScoreEvidenceSignals {
    MemoryGateScoreEvidenceSignals {
        react_revalidation_score: 0.91,
        graph_consistency_score: 0.88,
        omega_alignment_score: 0.93,
        current_state,
    }
}

#[test]
fn build_memory_gate_score_request_rows_from_evidence_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence = build_memory_gate_score_evidence_row_from_episode(
        &sample_episode("memory-alpha"),
        Some("searchinfra".to_string()),
        &gate_score_signals(MemoryLifecycleState::Active),
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
    assert!(row.ttl_score > 0.0);
    assert_eq!(row.current_state, "active");

    Ok(())
}

#[test]
fn build_memory_gate_score_request_batch_from_evidence_materializes_staged_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let evidence_rows = vec![
        build_memory_gate_score_evidence_row_from_episode(
            &sample_episode("memory-alpha"),
            Some("searchinfra".to_string()),
            &gate_score_signals(MemoryLifecycleState::Active),
        ),
        build_memory_gate_score_evidence_row_from_episode(
            &sample_episode("memory-beta"),
            None,
            &MemoryGateScoreEvidenceSignals {
                react_revalidation_score: 0.77,
                graph_consistency_score: 0.74,
                omega_alignment_score: 0.81,
                current_state: MemoryLifecycleState::Cooling,
            },
        ),
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

#[test]
fn build_memory_gate_score_evidence_row_from_store_roundtrips_real_episode()
-> Result<(), Box<dyn std::error::Error>> {
    let (_temp, store) = make_store()?;
    store.store(sample_episode("memory-alpha"))?;

    let evidence =
        build_memory_gate_score_evidence_row_from_store(MemoryGateScoreStoreEvidenceInput {
            store: &store,
            memory_id: "memory-alpha".into(),
            scenario_pack: Some("searchinfra".to_string()),
            signals: gate_score_signals(MemoryLifecycleState::RevalidatePending),
        })?;
    let rows = build_memory_gate_score_request_rows_from_evidence(&[evidence])?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].memory_id, "memory-alpha");
    assert_eq!(rows[0].usage_count, 6);
    assert_eq!(rows[0].current_state, "revalidate_pending");
    assert!((rows[0].failure_rate - (1.0 / 6.0)).abs() < 1e-6);

    Ok(())
}
