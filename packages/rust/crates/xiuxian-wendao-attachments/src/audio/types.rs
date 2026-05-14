//! Raw `audio` planning DTOs shared by Rust scheduling and Python adapters.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Stable schema marker for audio shard manifests.
pub const AUDIO_SHARD_MANIFEST_SCHEMA: &str = "xiuxian_wendao.audio_shards.v1";

/// Default audio shard profile shared by local and hosted backends.
pub const DEFAULT_AUDIO_SHARD_PROFILE: &str = "audio-shards-v1";

/// Audio shard offset selection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioShardStrategy {
    /// Select contiguous chunks from the requested start offset.
    Head,
    /// Spread a bounded number of chunks across the available duration.
    Uniform,
}

/// Raw DTO boundary and stringly state boundary for source audio identity rows.
///
/// Source audio metadata used to build deterministic shard identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceIdentity {
    /// Stable source id from the caller, usually a repository-relative path.
    pub source_id: String,
    /// SHA-256 of the source file bytes.
    pub source_sha256: String,
    /// Source duration in milliseconds when known.
    pub duration_ms: Option<u64>,
}

/// Raw DTO boundary and stringly state boundary for audio shard plan rows.
///
/// Audio shard plan input independent of a concrete model or backend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardPlan {
    /// Contract/profile id used for identity construction.
    pub profile: String,
    /// Input source identity.
    pub source: AudioSourceIdentity,
    /// Fixed shard duration in milliseconds.
    pub chunk_duration_ms: u64,
    /// Planned shard start offsets in milliseconds.
    pub start_offsets_ms: Vec<u64>,
    /// Context included before each logical shard when materializing media.
    pub context_before_ms: u64,
    /// Context included after each logical shard when materializing media.
    pub context_after_ms: u64,
    /// Output sample rate for normalized shard media.
    pub sample_rate_hz: u32,
    /// Number of output channels.
    pub channels: u8,
    /// Normalized audio container or codec token, such as `wav` or `flac`.
    pub audio_format: String,
    /// Shard selection strategy, such as `head` or `uniform`.
    pub strategy: String,
}

/// Raw DTO boundary and stringly state boundary for audio planner inputs.
///
/// Input for building an audio shard plan from source duration and strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardPlannerInput {
    /// Contract/profile id used for identity construction.
    pub profile: String,
    /// Input source identity.
    pub source: AudioSourceIdentity,
    /// Fixed logical shard duration in milliseconds.
    pub chunk_duration_ms: u64,
    /// Maximum number of chunks to materialize.
    pub limit_chunks: u32,
    /// Start offset for `head` and lower bound for `uniform`.
    pub start_offset_ms: u64,
    /// Selection strategy.
    pub strategy: AudioShardStrategy,
    /// Context included before each logical shard when materializing media.
    pub context_before_ms: u64,
    /// Context included after each logical shard when materializing media.
    pub context_after_ms: u64,
    /// Output sample rate for normalized shard media.
    pub sample_rate_hz: u32,
    /// Number of output channels.
    pub channels: u8,
    /// Normalized audio container or codec token, such as `wav` or `flac`.
    pub audio_format: String,
}

/// Raw DTO boundary and stringly state boundary for audio shard manifest rows.
///
/// One materialized audio shard addressed by a deterministic cache key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardManifestItem {
    /// Deterministic shard id independent of backend model selection.
    pub shard_id: String,
    /// Source id copied from the plan.
    pub source_id: String,
    /// Source file SHA-256 copied from the plan.
    pub source_sha256: String,
    /// Zero-based shard index in listening order.
    pub chunk_index: u32,
    /// Shard start offset in milliseconds.
    pub start_ms: u64,
    /// Shard duration in milliseconds.
    pub duration_ms: u64,
    /// Actual media start offset after applying bounded pre-context.
    pub media_start_ms: u64,
    /// Actual media duration after applying bounded pre/post-context.
    pub media_duration_ms: u64,
    /// Effective context included before the logical shard.
    pub context_before_ms: u64,
    /// Effective context included after the logical shard.
    pub context_after_ms: u64,
    /// Output sample rate for normalized shard media.
    pub sample_rate_hz: u32,
    /// Number of output channels.
    pub channels: u8,
    /// Normalized audio container or codec token.
    pub audio_format: String,
    /// Cache key for normalized shard media and downstream result reuse.
    pub cache_key: String,
    /// Listening-order key stable across backend retries.
    pub reading_order_key: String,
}

/// Raw DTO boundary and stringly state boundary for Rust audio media materialization.
///
/// The caller supplies executable and path choices while attachments owns
/// deterministic shard windows, output names, and cache identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioShardMaterializationInput {
    /// Source media path to pass to the local media splitter.
    pub source_path: PathBuf,
    /// Output directory for normalized shard media.
    pub output_dir: PathBuf,
    /// Media splitter executable, normally `ffmpeg`.
    pub ffmpeg_path: PathBuf,
    /// Recreate an existing shard file when true.
    pub force: bool,
}

/// Raw DTO boundary and stringly state boundary for one materialized audio shard.
///
/// The manifest row is the stable scheduling identity; `output_path` is the
/// local normalized media artifact for Python or hosted backends to consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioShardMaterializedItem {
    /// Stable manifest row for this shard.
    pub manifest: AudioShardManifestItem,
    /// Normalized shard media path created by Rust-side materialization.
    pub output_path: PathBuf,
}

/// Raw DTO boundary and stringly state boundary for audio result cache inputs.
///
/// Backend/task identity for caching downstream audio processing results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioResultCacheInput {
    /// Cache key from the normalized audio shard manifest.
    pub shard_cache_key: String,
    /// Logical task profile, such as transcription, diarization, or summarization.
    pub task_profile: String,
    /// Backend family or service id chosen by the scheduler.
    pub backend_id: String,
    /// Hash of backend configuration that affects output semantics.
    pub backend_config_hash: String,
}
