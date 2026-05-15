//! Unit tests for model-agnostic audio shard contracts.

use xiuxian_wendao_attachments::audio::{
    AudioResultCacheInput, AudioShardInput, AudioShardMaterializationInput, AudioShardPlan,
    AudioShardPlannerInput, AudioShardResult, AudioShardStrategy, AudioSourceIdentity,
    DEFAULT_AUDIO_SHARD_PROFILE, audio_result_cache_key, build_audio_shard_plan,
    materialize_audio_shards, merge_audio_shard_results, plan_audio_shards,
};

#[cfg(feature = "audio-shard-arrow")]
use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializedItem, AudioShardWorkerProfile, build_audio_shard_input_batch,
    build_audio_shard_inputs, build_audio_shard_result_batch, decode_audio_shard_result_batches,
};

fn sample_plan() -> AudioShardPlan {
    AudioShardPlan {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(90_000),
        },
        chunk_duration_ms: 30_000,
        start_offsets_ms: vec![0, 30_000, 60_000],
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "WAV".to_owned(),
        strategy: "uniform".to_owned(),
    }
}

fn planner_input() -> AudioShardPlannerInput {
    AudioShardPlannerInput {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(300_000),
        },
        chunk_duration_ms: 30_000,
        limit_chunks: 3,
        start_offset_ms: 10_000,
        strategy: AudioShardStrategy::Uniform,
        context_before_ms: 2_000,
        context_after_ms: 3_000,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
    }
}

#[test]
fn audio_shard_planner_builds_uniform_offsets_in_rust() -> Result<(), String> {
    let plan = build_audio_shard_plan(&planner_input())?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 140_000, 270_000]);
    assert_eq!(plan.strategy, "uniform");
    assert_eq!(plan.context_before_ms, 2_000);
    assert_eq!(plan.context_after_ms, 3_000);
    Ok(())
}

#[test]
fn audio_shard_planner_builds_head_offsets_in_rust() -> Result<(), String> {
    let mut input = planner_input();
    input.strategy = AudioShardStrategy::Head;

    let plan = build_audio_shard_plan(&input)?;

    assert_eq!(plan.start_offsets_ms, vec![10_000, 40_000, 70_000]);
    assert_eq!(plan.strategy, "head");
    Ok(())
}

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
    let first = sample_audio_input("first", "000001.000000030000");
    let second = sample_audio_input("second", "000000.000000000000");
    let inputs = vec![first.clone(), second.clone()];
    let first_result = AudioShardResult::succeeded(&first, "论坛开始，今天讨论行业趋势", 0.9);
    let second_result = AudioShardResult::succeeded(&second, "大家好，论坛开始", 0.9);

    let report = merge_audio_shard_results(&inputs, &[first_result, second_result])?;

    assert_eq!(report.text, "大家好，论坛开始，今天讨论行业趋势");
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
fn audio_materialization_runs_splitter_with_planned_media_windows() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    let log_path = tempdir.path().join("ffmpeg.log");
    std::fs::write(
        ffmpeg_path.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\ntouch \"$last\"\n",
            log_path.display()
        ),
    )
    .map_err(error_to_string)?;
    make_executable(ffmpeg_path.as_path())?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![30_000];
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;
    let input = AudioShardMaterializationInput {
        source_path: source_path.clone(),
        output_dir: tempdir.path().join("chunks"),
        ffmpeg_path,
        force: true,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert!(items[0].output_path.exists());
    assert_eq!(items[0].manifest.media_start_ms, 28_000);
    assert_eq!(items[0].manifest.media_duration_ms, 35_000);
    let log = std::fs::read_to_string(log_path).map_err(error_to_string)?;
    assert!(log.contains("-ss"));
    assert!(log.contains("28.000"));
    assert!(log.contains("-t"));
    assert!(log.contains("35.000"));
    assert!(log.contains(source_path.to_string_lossy().as_ref()));
    Ok(())
}

#[test]
fn audio_materialization_reuses_existing_chunks_without_splitter() -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(error_to_string)?;
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").map_err(error_to_string)?;
    let ffmpeg_path = tempdir.path().join("missing_ffmpeg");
    let output_dir = tempdir.path().join("chunks");
    std::fs::create_dir_all(output_dir.as_path()).map_err(error_to_string)?;
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let existing_manifest = plan_audio_shards(&plan)?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let existing_path = output_dir.join(format!(
        "audio_{:06}_{}.{}",
        existing_manifest.chunk_index,
        existing_manifest
            .shard_id
            .chars()
            .take(16)
            .collect::<String>(),
        existing_manifest.audio_format
    ));
    std::fs::write(existing_path.as_path(), b"cached").map_err(error_to_string)?;
    let input = AudioShardMaterializationInput {
        source_path,
        output_dir,
        ffmpeg_path,
        force: false,
    };

    let items = materialize_audio_shards(&plan, &input)?;

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].output_path, existing_path);
    Ok(())
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_shard_arrow_contract_roundtrips_results() -> Result<(), String> {
    let manifest = plan_audio_shards(&sample_plan())?
        .into_iter()
        .next()
        .ok_or_else(|| "expected one audio shard manifest".to_owned())?;
    let materialized = AudioShardMaterializedItem {
        manifest,
        output_path: std::path::PathBuf::from("/tmp/audio.wav"),
        shard_sha256: "b".repeat(64),
    };
    let profile = AudioShardWorkerProfile::transcription("hosted-audio");

    let inputs = build_audio_shard_inputs(&[materialized], &profile);
    let input_batch = build_audio_shard_input_batch(inputs.as_slice())?;
    let success = AudioShardResult::succeeded(&inputs[0], "transcript", 0.88);
    let result_batch = build_audio_shard_result_batch(std::slice::from_ref(&success))?;
    let decoded = decode_audio_shard_result_batches(&[result_batch])?;

    assert_eq!(input_batch.num_rows(), 1);
    assert_eq!(
        inputs[0].contract_version,
        "xiuxian_wendao.audio_shard_input.v1"
    );
    assert_eq!(inputs[0].task_profile, "transcription");
    assert_eq!(inputs[0].backend_profile, "hosted-audio");
    assert_eq!(decoded, vec![success]);
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(error_to_string)?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).map_err(error_to_string)
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn error_to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn sample_audio_input(shard_element_id: &str, reading_order_key: &str) -> AudioShardInput {
    AudioShardInput {
        contract_version: "xiuxian_wendao.audio_shard_input.v1".to_owned(),
        source_path: "/tmp/source.mp3".to_owned(),
        source_content_hash: "sourcehash".to_owned(),
        shard_path: format!("/tmp/{shard_element_id}.wav"),
        shard_sha256: format!("shardhash-{shard_element_id}"),
        shard_profile: "audio-shards-v1".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        preferred_languages: vec!["zh".to_owned()],
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        start_ms: 0,
        duration_ms: 30_000,
        media_start_ms: 0,
        media_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        shard_element_id: shard_element_id.to_owned(),
        reading_order_key: reading_order_key.to_owned(),
    }
}
