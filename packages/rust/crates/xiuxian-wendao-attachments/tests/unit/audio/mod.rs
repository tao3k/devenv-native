//! Unit tests for model-agnostic audio shard contracts.

pub(super) use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchCandidate, AudioRecoveryPatchDecisionKind, AudioRecoveryPatchGateOptions,
    AudioRecoveryPatchMergeRequest, AudioResultCacheInput, AudioRiskParentSelectionOptions,
    AudioShardInput, AudioShardMaterializationInput, AudioShardPlan, AudioShardPlannerInput,
    AudioShardRequestMetric, AudioShardResult, AudioShardResultStatus, AudioShardStrategy,
    AudioSourceIdentity, AudioSpeechSegment, AudioSpeechWindowPlannerInput,
    AudioTranscriptOrgLedgerOptions, DEFAULT_AUDIO_SHARD_PROFILE,
    apply_audio_recovery_patch_decisions, audio_result_cache_key,
    build_audio_recovery_patch_candidates, build_audio_recovery_speech_window_plan_for_inputs,
    build_audio_recovery_split_plan, build_audio_recovery_split_plan_for_inputs,
    build_audio_shard_plan, build_audio_speech_window_plan, build_audio_transcript_org_ledger,
    gate_audio_recovery_patches, materialize_audio_shards, merge_audio_shard_results,
    merge_audio_shard_results_with_recovery_patches, parse_audio_speech_segments_sidecar,
    plan_audio_shards, project_audio_transcript_org_evidence, select_audio_risk_parent_shards,
};

#[cfg(feature = "audio-shard-arrow")]
pub(super) use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializedItem, AudioShardWorkerProfile, build_audio_org_evidence_segment_batch,
    build_audio_org_evidence_source_batch, build_audio_shard_input_batch, build_audio_shard_inputs,
    build_audio_shard_result_batch, decode_audio_shard_result_batches,
};

#[cfg(feature = "audio-shard-arrow")]
mod arrow_contract;
mod materialize;
mod merge_ledger;
mod org_projection;
mod planning;
mod recovery_patch;
mod recovery_select;
mod speech_segments;

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
        window_durations_ms: Vec::new(),
        context_before_ms: 0,
        context_after_ms: 0,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "WAV".to_owned(),
        strategy: "uniform".to_owned(),
    }
}

fn speech_window_planner_input() -> AudioSpeechWindowPlannerInput {
    AudioSpeechWindowPlannerInput {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: "recordings/forum.mp3".to_owned(),
            source_sha256: "a".repeat(64),
            duration_ms: Some(300_000),
        },
        chunk_duration_ms: 30_000,
        limit_chunks: 16,
        speech_segments: vec![
            AudioSpeechSegment {
                index: 0,
                start_ms: 0,
                duration_ms: 4_000,
            },
            AudioSpeechSegment {
                index: 1,
                start_ms: 9_000,
                duration_ms: 3_000,
            },
            AudioSpeechSegment {
                index: 2,
                start_ms: 14_000,
                duration_ms: 3_000,
            },
            AudioSpeechSegment {
                index: 3,
                start_ms: 50_000,
                duration_ms: 45_000,
            },
        ],
        merge_gap_ms: 1_000,
        min_window_ms: 8_000,
        short_merge_gap_ms: Some(3_000),
        max_window_ms: Some(30_000),
        context_before_ms: 500,
        context_after_ms: 700,
        sample_rate_hz: 16_000,
        channels: 1,
        audio_format: "wav".to_owned(),
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
