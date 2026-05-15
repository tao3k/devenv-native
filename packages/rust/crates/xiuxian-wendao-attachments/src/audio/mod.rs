//! Model-agnostic audio shard planning and cache identity contracts.

#[cfg(feature = "audio-shard-arrow")]
mod batches;
mod cache;
mod identity;
mod materialize;
mod merge;
mod plan;
mod types;

#[cfg(feature = "audio-shard-arrow")]
pub use batches::{
    build_audio_shard_input_batch, build_audio_shard_inputs, build_audio_shard_result_batch,
    decode_audio_shard_result_batch, decode_audio_shard_result_batches,
};
pub use cache::audio_result_cache_key;
pub use materialize::materialize_audio_shards;
pub use merge::{AudioShardMergeReport, merge_audio_shard_results};
pub use plan::{build_audio_shard_plan, plan_audio_shards};
pub use types::{
    AUDIO_SHARD_INPUT_SCHEMA_VERSION, AUDIO_SHARD_MANIFEST_SCHEMA,
    AUDIO_SHARD_RESULT_SCHEMA_VERSION, AudioResultCacheInput, AudioShardInput,
    AudioShardManifestItem, AudioShardMaterializationInput, AudioShardMaterializedItem,
    AudioShardPlan, AudioShardPlannerInput, AudioShardResult, AudioShardResultStatus,
    AudioShardStrategy, AudioShardWorkerProfile, AudioSourceIdentity, DEFAULT_AUDIO_SHARD_PROFILE,
    DEFAULT_AUDIO_TASK_PROFILE,
};
