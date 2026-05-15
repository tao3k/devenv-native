//! Raw `audio` planning DTOs shared by Rust scheduling and Python adapters.

use super::identity::sha256_hex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Stable schema version for audio shard input batches.
pub const AUDIO_SHARD_INPUT_SCHEMA_VERSION: &str = "xiuxian_wendao.audio_shard_input.v1";
/// Stable schema version for audio shard result batches.
pub const AUDIO_SHARD_RESULT_SCHEMA_VERSION: &str = "xiuxian_wendao.audio_shard_result.v1";
/// Stable schema marker for audio shard manifests.
pub const AUDIO_SHARD_MANIFEST_SCHEMA: &str = "xiuxian_wendao.audio_shards.v1";

/// Default audio shard profile shared by local and hosted backends.
pub const DEFAULT_AUDIO_SHARD_PROFILE: &str = "audio-shards-v1";
/// Default logical task for speech-to-text style audio shard workers.
pub const DEFAULT_AUDIO_TASK_PROFILE: &str = "transcription";

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardMaterializedItem {
    /// Stable manifest row for this shard.
    pub manifest: AudioShardManifestItem,
    /// Normalized shard media path created by Rust-side materialization.
    pub output_path: PathBuf,
    /// SHA-256 of the normalized shard media bytes.
    pub shard_sha256: String,
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

/// Worker profile used to derive audio shard input rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardWorkerProfile {
    /// Logical task profile, such as transcription or summarization.
    pub task_profile: String,
    /// Backend family or service id selected by the Rust scheduler.
    pub backend_profile: String,
    /// Preferred language tags supplied to the worker.
    pub preferred_languages: Vec<String>,
}

impl AudioShardWorkerProfile {
    /// Create a model-neutral transcription worker profile.
    #[must_use]
    pub fn transcription(backend_profile: impl Into<String>) -> Self {
        Self {
            task_profile: DEFAULT_AUDIO_TASK_PROFILE.to_owned(),
            backend_profile: backend_profile.into(),
            preferred_languages: vec!["auto".to_owned()],
        }
    }
}

/// Raw DTO boundary for one audio shard worker input row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardInput {
    /// Stable schema version for the input row.
    pub contract_version: String,
    /// Source media path that produced the shard.
    pub source_path: String,
    /// SHA-256 of the original source media bytes.
    pub source_content_hash: String,
    /// Local normalized shard media path.
    pub shard_path: String,
    /// SHA-256 of the normalized shard media bytes.
    pub shard_sha256: String,
    /// Rust-side shard generation profile.
    pub shard_profile: String,
    /// Logical task profile requested from the worker.
    pub task_profile: String,
    /// Backend family or service id selected by the scheduler.
    pub backend_profile: String,
    /// Preferred language tags supplied to the worker.
    pub preferred_languages: Vec<String>,
    /// Normalized shard sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Normalized shard channel count.
    pub channels: u8,
    /// Normalized audio container or codec token.
    pub audio_format: String,
    /// Logical shard start offset in milliseconds.
    pub start_ms: u64,
    /// Logical shard duration in milliseconds.
    pub duration_ms: u64,
    /// Actual media start offset after applying bounded context.
    pub media_start_ms: u64,
    /// Actual media duration after applying bounded context.
    pub media_duration_ms: u64,
    /// Effective context included before the logical shard.
    pub context_before_ms: u64,
    /// Effective context included after the logical shard.
    pub context_after_ms: u64,
    /// Stable shard element id used for result correlation.
    pub shard_element_id: String,
    /// Stable listening-order key for deterministic merge.
    pub reading_order_key: String,
}

/// Stable audio worker result status values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioShardResultStatus {
    /// Worker completed the shard successfully.
    Succeeded,
    /// Worker failed the shard and reported an error.
    Failed,
    /// Worker intentionally skipped the shard.
    Skipped,
}

impl AudioShardResultStatus {
    /// Return the stable serialized status string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    /// Decode a stable audio result status value.
    ///
    /// # Errors
    ///
    /// Returns an error when the status is outside the stable result contract.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            other => Err(format!("unsupported audio shard result status `{other}`")),
        }
    }
}

/// Raw DTO boundary for one audio shard worker result row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioShardResult {
    /// Stable schema version for the result row.
    pub contract_version: String,
    /// Source media path copied from the input row.
    pub source_path: String,
    /// SHA-256 of the original source media bytes.
    pub source_content_hash: String,
    /// Local normalized shard media path copied from the input row.
    pub shard_path: String,
    /// SHA-256 of the normalized shard media bytes.
    pub shard_sha256: String,
    /// Rust-side shard generation profile.
    pub shard_profile: String,
    /// Logical task profile executed by the worker.
    pub task_profile: String,
    /// Backend family or service id used by the worker.
    pub backend_profile: String,
    /// Stable result status.
    pub status: AudioShardResultStatus,
    /// Recognized or generated text for successful rows.
    pub text: Option<String>,
    /// MIME type of `text`.
    pub text_mime_type: String,
    /// Optional worker confidence score.
    pub confidence: Option<f64>,
    /// Optional failure or skip reason.
    pub error_message: Option<String>,
    /// Stable shard element id copied from the input row.
    pub shard_element_id: String,
    /// Stable result element id used by downstream merge/caches.
    pub element_id: String,
}

impl AudioShardResult {
    /// Build a successful audio result for an input shard.
    #[must_use]
    pub fn succeeded(input: &AudioShardInput, text: impl Into<String>, confidence: f64) -> Self {
        Self::from_input(
            input,
            AudioShardResultStatus::Succeeded,
            Some(text.into()),
            Some(confidence),
            None,
        )
    }

    /// Build a failed audio result for an input shard.
    #[must_use]
    pub fn failed(input: &AudioShardInput, error_message: impl Into<String>) -> Self {
        Self::from_input(
            input,
            AudioShardResultStatus::Failed,
            None,
            None,
            Some(error_message.into()),
        )
    }

    /// Build a skipped audio result for an input shard.
    #[must_use]
    pub fn skipped(input: &AudioShardInput, reason: impl Into<String>) -> Self {
        Self::from_input(
            input,
            AudioShardResultStatus::Skipped,
            None,
            None,
            Some(reason.into()),
        )
    }

    fn from_input(
        input: &AudioShardInput,
        status: AudioShardResultStatus,
        text: Option<String>,
        confidence: Option<f64>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            contract_version: AUDIO_SHARD_RESULT_SCHEMA_VERSION.to_owned(),
            source_path: input.source_path.clone(),
            source_content_hash: input.source_content_hash.clone(),
            shard_path: input.shard_path.clone(),
            shard_sha256: input.shard_sha256.clone(),
            shard_profile: input.shard_profile.clone(),
            task_profile: input.task_profile.clone(),
            backend_profile: input.backend_profile.clone(),
            status,
            text,
            text_mime_type: "text/plain".to_owned(),
            confidence,
            error_message,
            shard_element_id: input.shard_element_id.clone(),
            element_id: audio_result_element_id(input),
        }
    }
}

fn audio_result_element_id(input: &AudioShardInput) -> String {
    sha256_hex(
        format!(
            "{}:{}:{}:{}:{}:{}",
            input.source_content_hash,
            input.shard_sha256,
            input.shard_profile,
            input.task_profile,
            input.backend_profile,
            input.shard_element_id
        )
        .as_bytes(),
    )
}
