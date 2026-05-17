use super::{
    AudioRecoveryPatchCandidate, AudioRecoveryPatchDecisionKind, AudioRecoveryPatchGateOptions,
    AudioRecoveryPatchMergeRequest, AudioShardResult, AudioShardResultStatus,
    apply_audio_recovery_patch_decisions, build_audio_recovery_patch_candidates,
    gate_audio_recovery_patches, merge_audio_shard_results_with_recovery_patches,
    sample_audio_input,
};

#[test]
fn audio_recovery_patch_gate_accepts_short_window_precision_gain() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery_a = sample_audio_input("recovery-a", "000000.000000000000");
    let recovery_b = sample_audio_input("recovery-b", "000001.000000030000");
    let base_result = AudioShardResult::succeeded(
        &parent,
        "测试测试测试测试测试测试测试测试测试测试通用会议",
        0.8,
    );
    let recovery_results = vec![
        AudioShardResult::succeeded(&recovery_a, "通用测试会议今天讨论流程", 0.9),
        AudioShardResult::succeeded(&recovery_b, "主持人介绍测试案例", 0.9),
    ];
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery-a".to_owned(), "recovery-b".to_owned()],
    }];

    let base_inputs = [parent];
    let base_results = [base_result];
    let (merge_report, gate_report) =
        merge_audio_shard_results_with_recovery_patches(AudioRecoveryPatchMergeRequest {
            base_inputs: &base_inputs,
            base_results: &base_results,
            recovery_results: recovery_results.as_slice(),
            candidates: candidates.as_slice(),
            options: AudioRecoveryPatchGateOptions::default(),
        })?;

    assert_eq!(gate_report.accepted_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::AcceptPatch
    );
    assert_eq!(
        merge_report.text,
        "通用测试会议今天讨论流程\n主持人介绍测试案例"
    );
    assert!(merge_report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_recovery_patch_decisions_build_final_base_rows_for_ledger() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery = sample_audio_input("recovery", "000000.000000000000");
    let base_result = AudioShardResult::failed(&parent, "audio transcript quality gate failed");
    let recovery_result = AudioShardResult::succeeded(&recovery, "recovered transcript", 0.9);
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery".to_owned()],
    }];
    let gate_report = gate_audio_recovery_patches(
        std::slice::from_ref(&base_result),
        &[recovery_result],
        &candidates,
        AudioRecoveryPatchGateOptions::default(),
    )?;

    let patched = apply_audio_recovery_patch_decisions(&[base_result], &gate_report);

    assert_eq!(patched.len(), 1);
    assert_eq!(patched[0].status, AudioShardResultStatus::Succeeded);
    assert_eq!(patched[0].text.as_deref(), Some("recovered transcript"));
    assert_eq!(patched[0].error_message, None);
    Ok(())
}

#[test]
fn audio_recovery_patch_gate_rejects_precision_regression() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery = sample_audio_input("recovery", "000000.000000000000");
    let base_result = AudioShardResult::succeeded(&parent, "通用测试会议讨论流程案例", 0.8);
    let recovery_result = AudioShardResult::succeeded(&recovery, "aaaaaa", 0.9);
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery".to_owned()],
    }];

    let gate_report = gate_audio_recovery_patches(
        &[base_result],
        &[recovery_result],
        candidates.as_slice(),
        AudioRecoveryPatchGateOptions::default(),
    )?;

    assert_eq!(gate_report.accepted_count, 0);
    assert_eq!(gate_report.rejected_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::RejectPatch
    );
    assert!(
        gate_report.decisions[0]
            .rejection_reasons
            .contains(&"chinese-ratio-drop".to_owned())
    );
    assert!(
        gate_report.decisions[0]
            .rejection_reasons
            .contains(&"char-collapse".to_owned())
    );
    Ok(())
}

#[test]
fn audio_recovery_patch_gate_accepts_recovery_for_failed_parent() -> Result<(), String> {
    let parent = sample_audio_input("parent", "000000.000000000000");
    let recovery = sample_audio_input("recovery", "000000.000000000000");
    let base_result = AudioShardResult::failed(&parent, "audio transcript quality gate failed");
    let recovery_result = AudioShardResult::succeeded(&recovery, "今天讨论家居行业供应链", 0.9);
    let candidates = vec![AudioRecoveryPatchCandidate {
        parent_shard_element_id: "parent".to_owned(),
        recovery_shard_element_ids: vec!["recovery".to_owned()],
    }];

    let base_inputs = [parent];
    let base_results = [base_result];
    let recovery_results = [recovery_result];
    let (merge_report, gate_report) =
        merge_audio_shard_results_with_recovery_patches(AudioRecoveryPatchMergeRequest {
            base_inputs: &base_inputs,
            base_results: &base_results,
            recovery_results: &recovery_results,
            candidates: candidates.as_slice(),
            options: AudioRecoveryPatchGateOptions::default(),
        })?;

    assert_eq!(gate_report.accepted_count, 1);
    assert_eq!(
        gate_report.decisions[0].decision,
        AudioRecoveryPatchDecisionKind::AcceptPatch
    );
    assert_eq!(merge_report.text, "今天讨论家居行业供应链");
    assert!(merge_report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_recovery_patch_candidates_group_recovery_windows_under_parent() -> Result<(), String> {
    let mut parent_a = sample_audio_input("parent-a", "000000.000000000000");
    parent_a.start_ms = 0;
    parent_a.duration_ms = 60_000;
    let mut parent_b = sample_audio_input("parent-b", "000001.000000060000");
    parent_b.start_ms = 60_000;
    parent_b.duration_ms = 60_000;
    let mut recovery_b = sample_audio_input("recovery-b", "000003.000000090000");
    recovery_b.start_ms = 90_000;
    recovery_b.duration_ms = 30_000;
    let mut recovery_a = sample_audio_input("recovery-a", "000002.000000030000");
    recovery_a.start_ms = 30_000;
    recovery_a.duration_ms = 30_000;

    let candidates =
        build_audio_recovery_patch_candidates(&[parent_b, parent_a], &[recovery_b, recovery_a])?;

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].parent_shard_element_id, "parent-a");
    assert_eq!(candidates[0].recovery_shard_element_ids, vec!["recovery-a"]);
    assert_eq!(candidates[1].parent_shard_element_id, "parent-b");
    assert_eq!(candidates[1].recovery_shard_element_ids, vec!["recovery-b"]);
    Ok(())
}

#[test]
fn audio_recovery_patch_candidates_reject_unowned_windows() -> Result<(), String> {
    let mut parent = sample_audio_input("parent", "000000.000000000000");
    parent.start_ms = 0;
    parent.duration_ms = 60_000;
    let mut recovery = sample_audio_input("recovery", "000002.000000120000");
    recovery.start_ms = 120_000;
    recovery.duration_ms = 30_000;

    let Err(error) = build_audio_recovery_patch_candidates(&[parent], &[recovery]) else {
        return Err("unowned recovery window unexpectedly mapped".to_owned());
    };

    assert!(error.contains("no parent logical window"));
    Ok(())
}
