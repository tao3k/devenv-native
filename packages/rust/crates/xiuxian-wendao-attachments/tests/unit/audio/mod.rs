//! Unit tests for model-agnostic audio shard contracts.

use xiuxian_wendao_attachments::audio::{
    AudioResultCacheInput, AudioShardMaterializationInput, AudioShardPlan, AudioShardPlannerInput,
    AudioShardStrategy, AudioSourceIdentity, DEFAULT_AUDIO_SHARD_PROFILE, audio_result_cache_key,
    build_audio_shard_plan, materialize_audio_shards, plan_audio_shards,
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
fn audio_shard_planner_builds_uniform_offsets_in_rust() {
    let plan = build_audio_shard_plan(&planner_input()).expect("valid input");

    assert_eq!(plan.start_offsets_ms, vec![10_000, 140_000, 270_000]);
    assert_eq!(plan.strategy, "uniform");
    assert_eq!(plan.context_before_ms, 2_000);
    assert_eq!(plan.context_after_ms, 3_000);
}

#[test]
fn audio_shard_planner_builds_head_offsets_in_rust() {
    let mut input = planner_input();
    input.strategy = AudioShardStrategy::Head;

    let plan = build_audio_shard_plan(&input).expect("valid input");

    assert_eq!(plan.start_offsets_ms, vec![10_000, 40_000, 70_000]);
    assert_eq!(plan.strategy, "head");
}

#[test]
fn audio_shard_manifest_is_backend_independent_and_ordered() {
    let items = plan_audio_shards(&sample_plan()).expect("valid plan");

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].source_id, "recordings/forum.mp3");
    assert_eq!(items[0].audio_format, "wav");
    assert_eq!(items[0].reading_order_key, "000000.000000000000");
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].media_duration_ms, 30_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
    assert_ne!(items[0].shard_id, items[1].shard_id);
    assert!(items[0].cache_key.starts_with(DEFAULT_AUDIO_SHARD_PROFILE));
}

#[test]
fn audio_shard_identity_changes_with_precision_affecting_parameters() {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan).expect("valid plan");
    let mut changed = plan;
    changed.sample_rate_hz = 8_000;

    let changed_items = plan_audio_shards(&changed).expect("valid plan");

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
}

#[test]
fn audio_shard_media_window_preserves_logical_order_with_context() {
    let mut plan = sample_plan();
    plan.context_before_ms = 2_000;
    plan.context_after_ms = 3_000;

    let items = plan_audio_shards(&plan).expect("valid plan");

    assert_eq!(items[0].start_ms, 0);
    assert_eq!(items[0].media_start_ms, 0);
    assert_eq!(items[0].context_before_ms, 0);
    assert_eq!(items[0].context_after_ms, 3_000);
    assert_eq!(items[1].start_ms, 30_000);
    assert_eq!(items[1].media_start_ms, 28_000);
    assert_eq!(items[1].media_duration_ms, 35_000);
    assert_eq!(items[1].reading_order_key, "000001.000000030000");
}

#[test]
fn audio_shard_identity_changes_with_context() {
    let plan = sample_plan();
    let baseline = plan_audio_shards(&plan).expect("valid plan");
    let mut changed = plan;
    changed.context_after_ms = 1_000;

    let changed_items = plan_audio_shards(&changed).expect("valid plan");

    assert_ne!(baseline[0].shard_id, changed_items[0].shard_id);
}

#[test]
fn audio_shard_plan_rejects_invalid_contract_inputs() {
    let mut plan = sample_plan();
    plan.chunk_duration_ms = 0;

    let error = plan_audio_shards(&plan).expect_err("invalid duration");

    assert!(error.contains("chunk duration"));
}

#[test]
fn audio_result_cache_key_includes_backend_and_task_identity() {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: "hosted-audio".to_owned(),
        backend_config_hash: "model-a".to_owned(),
    };
    let baseline = audio_result_cache_key(&input).expect("valid cache input");
    let mut changed = input;
    changed.backend_config_hash = "model-b".to_owned();

    let changed_key = audio_result_cache_key(&changed).expect("valid cache input");

    assert!(baseline.starts_with("transcription:hosted-audio:"));
    assert_ne!(baseline, changed_key);
}

#[test]
fn audio_result_cache_key_rejects_empty_backend_identity() {
    let input = AudioResultCacheInput {
        shard_cache_key: "audio-shards-v1:abc".to_owned(),
        task_profile: "transcription".to_owned(),
        backend_id: "".to_owned(),
        backend_config_hash: "model-a".to_owned(),
    };

    let error = audio_result_cache_key(&input).expect_err("invalid backend");

    assert!(error.contains("backend id"));
}

#[test]
fn audio_materialization_runs_splitter_with_planned_media_windows() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").expect("source");
    let ffmpeg_path = tempdir.path().join("fake_ffmpeg.sh");
    let log_path = tempdir.path().join("ffmpeg.log");
    std::fs::write(
        ffmpeg_path.as_path(),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nlast=''\nfor arg in \"$@\"; do last=\"$arg\"; done\ntouch \"$last\"\n",
            log_path.display()
        ),
    )
    .expect("fake ffmpeg");
    make_executable(ffmpeg_path.as_path());
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

    let items = materialize_audio_shards(&plan, &input).expect("materialized");

    assert_eq!(items.len(), 1);
    assert!(items[0].output_path.exists());
    assert_eq!(items[0].manifest.media_start_ms, 28_000);
    assert_eq!(items[0].manifest.media_duration_ms, 35_000);
    let log = std::fs::read_to_string(log_path).expect("log");
    assert!(log.contains("-ss"));
    assert!(log.contains("28.000"));
    assert!(log.contains("-t"));
    assert!(log.contains("35.000"));
    assert!(log.contains(source_path.to_string_lossy().as_ref()));
}

#[test]
fn audio_materialization_reuses_existing_chunks_without_splitter() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let source_path = tempdir.path().join("source.mp3");
    std::fs::write(source_path.as_path(), b"source").expect("source");
    let ffmpeg_path = tempdir.path().join("missing_ffmpeg");
    let output_dir = tempdir.path().join("chunks");
    std::fs::create_dir_all(output_dir.as_path()).expect("chunks");
    let mut plan = sample_plan();
    plan.start_offsets_ms = vec![0];
    let existing_manifest = plan_audio_shards(&plan).expect("manifest")[0].clone();
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
    std::fs::write(existing_path.as_path(), b"cached").expect("cached");
    let input = AudioShardMaterializationInput {
        source_path,
        output_dir,
        ffmpeg_path,
        force: false,
    };

    let items = materialize_audio_shards(&plan, &input).expect("reused");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].output_path, existing_path);
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("permissions");
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) {}
