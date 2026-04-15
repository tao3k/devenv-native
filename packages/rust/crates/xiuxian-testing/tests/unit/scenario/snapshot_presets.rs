use super::*;

#[test]
fn snapshot_policy_recommended_enables_rich_metadata() {
    let policy = ScenarioSnapshotPolicy::recommended();

    assert!(policy.sort_maps());
    assert!(policy.include_description());
    assert!(policy.include_info());
    assert!(policy.include_input_file());
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::normalize_path(".**.path"))
    );
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::replace(
                ".**.request_id",
                "[request-id]"
            ))
    );
    assert!(
        !policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::round(".**.latency_ms", 2))
    );
}

#[test]
fn snapshot_policy_portable_ci_disables_input_file_metadata() {
    let policy = ScenarioSnapshotPolicy::portable_ci();

    assert!(policy.sort_maps());
    assert!(policy.include_description());
    assert!(policy.include_info());
    assert!(!policy.include_input_file());
}

#[test]
fn snapshot_policy_runtime_heavy_adds_timing_redactions() {
    let policy = ScenarioSnapshotPolicy::runtime_heavy();

    assert!(policy.include_input_file());
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::round(".**.latency_ms", 2))
    );
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::round(".**.duration_secs", 4))
    );
}

#[test]
fn snapshot_policy_supports_redaction_builders() {
    let mut policy = ScenarioSnapshotPolicy::new();
    policy
        .add_redaction(ScenarioSnapshotRedaction::replace(".request.id", "[id]"))
        .add_redaction(ScenarioSnapshotRedaction::sort(".flags"))
        .add_redaction(ScenarioSnapshotRedaction::round(".timings.latency_ms", 2))
        .add_redaction(ScenarioSnapshotRedaction::normalize_path(
            ".artifacts.output_path",
        ));

    assert_eq!(policy.redactions().len(), 4);
    assert_eq!(
        policy.redactions()[0],
        ScenarioSnapshotRedaction::Replace {
            selector: ".request.id".to_string(),
            replacement: "[id]".to_string(),
        }
    );
    assert_eq!(
        policy.redactions()[1],
        ScenarioSnapshotRedaction::Sort {
            selector: ".flags".to_string(),
        }
    );
    assert_eq!(
        policy.redactions()[2],
        ScenarioSnapshotRedaction::Round {
            selector: ".timings.latency_ms".to_string(),
            decimals: 2,
        }
    );
    assert_eq!(
        policy.redactions()[3],
        ScenarioSnapshotRedaction::NormalizePath {
            selector: ".artifacts.output_path".to_string(),
        }
    );
}

#[test]
fn snapshot_policy_supports_redaction_presets() {
    let mut policy = ScenarioSnapshotPolicy::new();
    policy
        .add_redaction_preset(ScenarioSnapshotRedactionPreset::portable_paths())
        .add_redaction_preset(ScenarioSnapshotRedactionPreset::runtime_volatility())
        .add_redaction_preset(ScenarioSnapshotRedactionPreset::timing_noise());

    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::normalize_path(".**.temp_dir"))
    );
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::replace(
                ".**.started_at",
                "[started-at]"
            ))
    );
    assert!(
        policy
            .redactions()
            .contains(&ScenarioSnapshotRedaction::round(".**.elapsed_ms", 2))
    );
}
