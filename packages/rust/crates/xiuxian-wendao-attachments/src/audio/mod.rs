//! Model-agnostic audio shard planning and cache identity contracts.

mod cache;
mod identity;
mod materialize;
mod plan;
mod types;

pub use cache::audio_result_cache_key;
pub use materialize::materialize_audio_shards;
pub use plan::{build_audio_shard_plan, plan_audio_shards};
pub use types::{
    AUDIO_SHARD_MANIFEST_SCHEMA, AudioResultCacheInput, AudioShardManifestItem,
    AudioShardMaterializationInput, AudioShardMaterializedItem, AudioShardPlan,
    AudioShardPlannerInput, AudioShardStrategy, AudioSourceIdentity, DEFAULT_AUDIO_SHARD_PROFILE,
};
