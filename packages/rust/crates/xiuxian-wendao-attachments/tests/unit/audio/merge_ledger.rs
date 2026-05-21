use super::{
    AudioResultCacheInput, AudioShardResult, AudioTranscriptOrgLedgerOptions,
    DEFAULT_AUDIO_SHARD_PROFILE, audio_result_cache_key, build_audio_transcript_org_ledger,
    error_to_string, merge_audio_shard_results, plan_audio_shards, sample_audio_input, sample_plan,
};

#[test]
fn audio_shard_manifest_is_backend_independent_and_ordered() -> Result<(), String> {
    let items = plan_audio_shards(&sample_plan())?;

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].source_id, "recordings/forum.mp3");
    assert_eq!(items[0].audio_format, "wav");
    assert_eq!(items[0].reading_order_key, "000000.000000000000");
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].media_duration_ms, 30_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
    assert_ne!(items[0].shard_id, items[1].shard_id);
    assert!(items[0].cache_key.starts_with(DEFAULT_AUDIO_SHARD_PROFILE));
    Ok(())
}

#[test]
fn audio_shard_identity_changes_with_precision_affecting_parameters() -> Result<(), String> {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan)?;
    let mut changed = plan;
    changed.sample_rate_hz = 8_000;

    let changed_items = plan_audio_shards(&changed)?;

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
    Ok(())
}

#[test]
fn audio_shard_media_window_preserves_logical_order_with_context() -> Result<(), String> {
    let mut plan = sample_plan();
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;

    let items = plan_audio_shards(&plan)?;

    assert_eq!(items[0].start_ms, 0);
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].context_before_ms, 0);
    assert_eq!(items[0].context_after_ms, 3_000);
    assert_eq!(items[1].start_ms, 30_000);
    assert_eq!(items[1].media_start_ms, 28_000);
    assert_eq!(items[1].media_duration_ms, 35_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
    Ok(())
}

#[test]
fn audio_shard_identity_changes_with_context() -> Result<(), String> {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan)?;
    let mut changed = plan;
    changed.context_after_ms = 1_000;

    let changed_items = plan_audio_shards(&changed)?;

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
    Ok(())
}

#[test]
fn audio_shard_plan_rejects_invalid_contract_inputs() -> Result<(), String> {
    let mut plan = sample_plan();
    plan.chunk_duration_ms = 0;

    let Err(error) = plan_audio_shards(&plan) else {
        return Err("invalid audio shard plan unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("chunk duration"));
    Ok(())
}

#[test]
fn audio_result_cache_key_includes_backend_and_task_identity() -> Result<(), String> {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: "hosted-audio".to_owned(),
        backend_config_hash: "model-a".to_owned(),
    };
    let baseline = audio_result_cache_key(&input)?;
    let mut changed = input;
    changed.backend_config_hash = "model-b".to_owned();

    let changed_key = audio_result_cache_key(&changed)?;

    assert!(baseline.starts_with("transcription:hosted-audio:"));
    assert_ne!(baseline, changed_key);
    Ok(())
}

#[test]
fn audio_result_cache_key_rejects_empty_backend_identity() -> Result<(), String> {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: String::new(),
        backend_config_hash: "model-a".to_owned(),
    };

    let Err(error) = audio_result_cache_key(&input) else {
        return Err("invalid backend identity unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("backend id"));
    Ok(())
}

#[test]
fn audio_shard_result_merge_preserves_reading_order_and_dedupes_boundary() -> Result<(), String> {
    let mut first = sample_audio_input("first", "000001.000000030000");
    let mut second = sample_audio_input("second", "000000.000000000000");
    first.start_ms = 30_000;
    second.start_ms = 0;
    let inputs = vec![first.clone(), second.clone()];
    let first_result = AudioShardResult::succeeded(&first, "论坛开始，今天讨论行业趋势", 0.9);
    let second_result = AudioShardResult::succeeded(&second, "大家好，论坛开始", 0.9);

    let report = merge_audio_shard_results(&inputs, &[first_result, second_result])?;

    assert_eq!(report.text, "大家好，论坛开始，今天讨论行业趋势");
    assert_eq!(
        report.timeline_text,
        "[00:00.000-00:30.000] 大家好，论坛开始\n[00:30.000-01:00.000] ，今天讨论行业趋势"
    );
    assert_eq!(report.succeeded_count, 2);
    assert!(report.has_complete_success_coverage());
    Ok(())
}

#[test]
fn audio_shard_result_merge_reports_failed_skipped_missing_and_duplicate_rows() -> Result<(), String>
{
    let first = sample_audio_input("first", "000000.000000000000");
    let second = sample_audio_input("second", "000001.000000030000");
    let third = sample_audio_input("third", "000002.000000060000");
    let duplicate = AudioShardResult::succeeded(&first, "duplicate", 0.9);
    let results = vec![
        AudioShardResult::failed(&first, "model failed"),
        duplicate.clone(),
        duplicate,
        AudioShardResult::skipped(&second, "not configured"),
    ];

    let report = merge_audio_shard_results(&[first, second, third], &results)?;

    assert!(!report.has_complete_success_coverage());
    assert_eq!(report.failed_shard_element_ids, vec!["first"]);
    assert_eq!(report.skipped_shard_element_ids, vec!["second"]);
    assert_eq!(report.missing_shard_element_ids, vec!["third"]);
    assert_eq!(report.duplicate_shard_element_ids, vec!["first"]);
    Ok(())
}

#[test]
fn audio_shard_result_merge_rejects_hash_mismatch() -> Result<(), String> {
    let input = sample_audio_input("first", "000000.000000000000");
    let mut result = AudioShardResult::succeeded(&input, "text", 0.9);
    result.shard_sha256 = "different".to_owned();

    let Err(error) = merge_audio_shard_results(&[input], &[result]) else {
        return Err("hash mismatch should be rejected".to_owned());
    };

    assert!(error.contains("shard hash mismatch"));
    Ok(())
}

#[test]
fn audio_transcript_org_ledger_uses_org_attachment_links() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let shard_dir = tempdir.path().join("audio_shards");
    std::fs::create_dir_all(&shard_dir).map_err(error_to_string)?;
    let first_path = shard_dir.join("audio_000000_first.wav");
    let second_path = shard_dir.join("audio_000001_second.wav");
    std::fs::write(&first_path, b"first").map_err(error_to_string)?;
    std::fs::write(&second_path, b"second").map_err(error_to_string)?;

    let mut first = sample_audio_input("first", "000000.000000000000");
    first.shard_path = first_path.to_string_lossy().into_owned();
    let mut second = sample_audio_input("second", "000001.000000030000");
    second.shard_path = second_path.to_string_lossy().into_owned();
    second.start_ms = 30_000;
    let results = vec![
        AudioShardResult::succeeded(&first, "First audio segment.", 0.91),
        AudioShardResult::succeeded(&second, "Second audio segment.", 0.92),
    ];

    let ledger = build_audio_transcript_org_ledger(
        &[first, second],
        &results,
        &AudioTranscriptOrgLedgerOptions::new("Forum transcript", "audio_shards"),
    )?;

    assert!(ledger.contains(":WENDAO_SCHEMA: xiuxian_wendao.audio_transcript_org_ledger.v1"));
    assert!(ledger.contains(":DIR: audio_shards"));
    assert!(ledger.contains("[[attachment:audio_000000_first.wav][normalized audio shard]]"));
    assert!(ledger.contains("*** 00:00:00.000 -- 00:00:30.000 source.mp3 shard 000000"));
    assert!(ledger.contains(":START_SECONDS: 0.000"));
    assert!(ledger.contains("First audio segment."));
    assert!(!ledger.contains("** Merged Transcript"));
    assert!(ledger.contains(":WENDAO_SHARD_ELEMENT_ID: second"));

    let lint_options = orgize::lint::LintOptions {
        attachment_base_dir: Some(tempdir.path().to_path_buf()),
        ..orgize::lint::LintOptions::default()
    };
    let report = orgize::lint::lint_org_with_options(&ledger, &lint_options);
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.code != "ORG016"),
        "attachment lint should be clean: {:#?}",
        report.findings
    );

    let parsed = orgize::Org::parse(&ledger).document();
    let attachment_links = parsed
        .section_index_records()
        .into_iter()
        .flat_map(|section| section.links)
        .filter(|link| link.attachment.is_some())
        .count();
    assert_eq!(attachment_links, 2);
    Ok(())
}

#[test]
fn audio_transcript_org_ledger_rejects_incomplete_success_coverage() -> Result<(), String> {
    let first = sample_audio_input("first", "000000.000000000000");
    let result = AudioShardResult::failed(&first, "model failed");

    let Err(error) = build_audio_transcript_org_ledger(
        std::slice::from_ref(&first),
        &[result],
        &AudioTranscriptOrgLedgerOptions::default(),
    ) else {
        return Err("incomplete audio ledger coverage unexpectedly succeeded".to_owned());
    };

    assert!(error.contains("complete success coverage"));
    Ok(())
}
