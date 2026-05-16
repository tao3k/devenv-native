//! Model-agnostic audio shard planning and cache identity contracts.

#[cfg(feature = "audio-shard-arrow")]
mod batches;
mod cache;
mod identity;
mod materialize;
mod merge;
mod org_ledger;
mod plan;
mod recovery_patch;
mod recovery_select;
mod speech_segments;
mod types;

#[cfg(feature = "audio-shard-arrow")]
pub use batches::{
    build_audio_shard_input_batch, build_audio_shard_inputs, build_audio_shard_result_batch,
    decode_audio_shard_result_batch, decode_audio_shard_result_batches,
};
pub use cache::audio_result_cache_key;
pub use materialize::materialize_audio_shards;
pub use merge::{AudioShardMergeReport, merge_audio_shard_results};
pub use org_ledger::{
    AUDIO_TRANSCRIPT_ORG_LEDGER_SCHEMA, AudioTranscriptOrgLedgerOptions,
    build_audio_transcript_org_ledger,
};
pub use plan::{
    build_audio_recovery_speech_window_plan_for_inputs, build_audio_recovery_split_plan,
    build_audio_recovery_split_plan_for_inputs, build_audio_shard_plan,
    build_audio_speech_window_plan, plan_audio_shards,
};
pub use recovery_patch::{
    AudioRecoveryPatchCandidate, AudioRecoveryPatchDecision, AudioRecoveryPatchDecisionKind,
    AudioRecoveryPatchGateOptions, AudioRecoveryPatchGateReport, AudioRecoveryPatchTextMetrics,
    apply_audio_recovery_patch_decisions, build_audio_recovery_patch_candidates,
    gate_audio_recovery_patches, merge_audio_shard_results_with_recovery_patches,
};
pub use recovery_select::{
    AudioRiskParentSelection, AudioRiskParentSelectionOptions, AudioShardRequestMetric,
    select_audio_risk_parent_shards,
};
pub use speech_segments::parse_audio_speech_segments_sidecar;
pub use types::{
    AUDIO_SHARD_INPUT_SCHEMA_VERSION, AUDIO_SHARD_MANIFEST_SCHEMA,
    AUDIO_SHARD_RESULT_SCHEMA_VERSION, AudioResultCacheInput, AudioShardInput,
    AudioShardManifestItem, AudioShardMaterializationInput, AudioShardMaterializedItem,
    AudioShardPlan, AudioShardPlannerInput, AudioShardResult, AudioShardResultStatus,
    AudioShardStrategy, AudioShardWorkerProfile, AudioSourceIdentity, AudioSpeechSegment,
    AudioSpeechWindowPlannerInput, DEFAULT_AUDIO_SHARD_PROFILE, DEFAULT_AUDIO_TASK_PROFILE,
};
