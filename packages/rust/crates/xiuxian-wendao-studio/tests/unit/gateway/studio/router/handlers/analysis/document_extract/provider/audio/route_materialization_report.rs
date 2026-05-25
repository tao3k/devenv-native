use std::path::PathBuf;

use xiuxian_qianji::{WorkflowStageFacts, WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace};
use xiuxian_wendao_attachments::audio::AudioShardManifestItem;

use super::{
    AUDIO_MATERIALIZATION_REPORT_NAME, AUDIO_MATERIALIZATION_REPORT_SCHEMA,
    AudioDocumentExtractConfig, AudioShardMaterializationSource, AudioShardMaterializedItem,
    write_audio_materialization_report,
};

#[test]
fn audio_materialization_report_records_artifact_backend_and_byte_sources() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp.path().join("out");
    std::fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    let artifact_root = temp.path().join("artifact-cache");
    let media_path = output_dir.join("base.wav");
    let cache_path = output_dir.join("recovery.wav");
    std::fs::write(media_path.as_path(), b"media").map_err(|error| error.to_string())?;
    std::fs::write(cache_path.as_path(), b"cache").map_err(|error| error.to_string())?;
    let config = AudioDocumentExtractConfig {
        backend_profile: "hosted-audio-transcript-v1".to_owned(),
        chunk_duration_ms: 30_000,
        context_before_ms: 0,
        context_after_ms: 0,
        recovery_split_duration_ms: 15_000,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        ffprobe_path: PathBuf::from("ffprobe"),
        artifact_cache_dir: Some(artifact_root.clone()),
        transcript_admission_dir: None,
        base_worker_budget: None,
        recovery_worker_budget: None,
        speech_segments_jsonl_path: None,
        speech_merge_gap_ms: 500,
        speech_min_window_ms: 0,
        speech_limit_chunks: 10_000,
    };
    let base = materialized_item(
        "base",
        media_path.clone(),
        AudioShardMaterializationSource::MediaSplitter,
    );
    let recovery = materialized_item(
        "recovery",
        cache_path.clone(),
        AudioShardMaterializationSource::ArtifactCache,
    );
    std::fs::remove_file(media_path.as_path()).map_err(|error| error.to_string())?;
    std::fs::remove_file(cache_path.as_path()).map_err(|error| error.to_string())?;

    let workflow_trace = WorkflowTrace {
        workflow_id: "audio.materialization.test".to_owned(),
        stages: vec![
            workflow_stage("audio.base.materialize_shards", 2_500_000),
            workflow_stage("audio.base.invoke_worker", 7_750_000),
        ],
    };

    write_audio_materialization_report(
        &output_dir,
        &config,
        &[base],
        &[recovery],
        &workflow_trace,
    )?;

    let report_path = output_dir.join(AUDIO_MATERIALIZATION_REPORT_NAME);
    let report = std::fs::read_to_string(report_path.as_path()).map_err(|error| {
        format!(
            "read audio materialization report `{}`: {error}",
            report_path.display()
        )
    })?;
    let value = serde_json::from_str::<serde_json::Value>(report.as_str())
        .map_err(|error| format!("parse audio materialization report: {error}"))?;
    assert_eq!(
        value.get("schema").and_then(serde_json::Value::as_str),
        Some(AUDIO_MATERIALIZATION_REPORT_SCHEMA)
    );
    assert_eq!(
        value
            .pointer("/artifactCache/configured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        value
            .pointer("/artifactCache/root")
            .and_then(serde_json::Value::as_str),
        Some(artifact_root.to_string_lossy().as_ref())
    );
    assert!(
        value
            .pointer("/artifactCache/backend")
            .and_then(serde_json::Value::as_str)
            .is_some()
    );
    assert!(
        value
            .pointer("/artifactCache/runtimeWorkers")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|workers| workers > 0)
    );
    assert!(
        value
            .pointer("/artifactCache/memoryShards")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|shards| shards > 0)
    );
    assert!(
        value
            .pointer("/artifactCache/recoverConcurrency")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|lanes| lanes > 0)
    );
    assert!(
        value
            .pointer("/artifactCache/flushers")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|lanes| lanes > 0)
    );
    assert!(
        value
            .pointer("/artifactCache/reclaimers")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|lanes| lanes > 0)
    );
    if value
        .pointer("/artifactCache/backend")
        .and_then(serde_json::Value::as_str)
        == Some("foyer")
    {
        assert_eq!(
            value
                .pointer("/artifactCache/memoryWeighter")
                .and_then(serde_json::Value::as_str),
            Some("bytes")
        );
        assert_eq!(
            value
                .pointer("/artifactCache/policy")
                .and_then(serde_json::Value::as_str),
            Some("write-on-insertion")
        );
        assert_eq!(
            value
                .pointer("/artifactCache/blockSizeBytes")
                .and_then(serde_json::Value::as_u64),
            Some(16 * 1024 * 1024)
        );
    }
    assert_eq!(
        value.get("byteCount").and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(
        value
            .pointer("/sourceBytes/media-splitter")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        value
            .pointer("/sourceBytes/artifact-cache")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        value
            .pointer("/workflow/workflowId")
            .and_then(serde_json::Value::as_str),
        Some("audio.materialization.test")
    );
    assert_eq!(
        value
            .pointer("/workflow/stageCount")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        value
            .pointer("/workflow/stageElapsedMs/audio.base.materialize_shards")
            .and_then(serde_json::Value::as_f64),
        Some(2.5)
    );
    assert_eq!(
        value
            .pointer("/workflow/stageElapsedMs/audio.base.invoke_worker")
            .and_then(serde_json::Value::as_f64),
        Some(7.75)
    );
    assert_eq!(
        value
            .pointer("/workflow/totalElapsedMs")
            .and_then(serde_json::Value::as_f64),
        Some(10.25)
    );
    Ok(())
}

fn workflow_stage(stage_id: &str, duration_nanos: u64) -> WorkflowStageTrace {
    WorkflowStageTrace {
        stage_id: stage_id.to_owned(),
        status: WorkflowStageStatus::Succeeded,
        started_unix_ms: 0,
        duration_nanos,
        input: WorkflowStageFacts::default(),
        output: WorkflowStageFacts::default(),
        error: None,
        checkpoints: Vec::new(),
    }
}

fn materialized_item(
    id: &str,
    output_path: PathBuf,
    materialization_source: AudioShardMaterializationSource,
) -> AudioShardMaterializedItem {
    AudioShardMaterializedItem {
        manifest: AudioShardManifestItem {
            shard_id: id.to_owned(),
            source_id: "source".to_owned(),
            source_sha256: "source-sha256".to_owned(),
            chunk_index: 0,
            start_ms: 0,
            duration_ms: 30_000,
            media_start_ms: 0,
            media_duration_ms: 30_000,
            context_before_ms: 0,
            context_after_ms: 0,
            sample_rate_hz: 16_000,
            channels: 1,
            audio_format: "wav".to_owned(),
            cache_key: format!("cache-{id}"),
            reading_order_key: format!("000000000000-{id}"),
        },
        output_path,
        shard_sha256: format!("sha256-{id}"),
        shard_byte_len: 5,
        materialization_source,
    }
}
