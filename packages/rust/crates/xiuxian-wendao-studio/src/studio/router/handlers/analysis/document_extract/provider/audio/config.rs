//! Model-neutral audio route configuration.

use std::path::PathBuf;

pub(super) const AUDIO_BACKEND_PROFILE_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_BACKEND_PROFILE";
pub(super) const AUDIO_CHUNK_MS_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHUNK_MS";
pub(super) const AUDIO_CONTEXT_BEFORE_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_BEFORE_MS";
pub(super) const AUDIO_CONTEXT_AFTER_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_CONTEXT_AFTER_MS";
pub(super) const AUDIO_RECOVERY_SPLIT_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_SPLIT_MS";
pub(super) const AUDIO_SAMPLE_RATE_HZ_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_SAMPLE_RATE_HZ";
pub(super) const AUDIO_CHANNELS_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_CHANNELS";
pub(super) const AUDIO_FORMAT_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_FORMAT";
pub(super) const AUDIO_FFMPEG_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFMPEG";
pub(super) const AUDIO_FFPROBE_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_FFPROBE";
pub(super) const AUDIO_BASE_WORKERS_ENV: &str = "WENDAO_DOCUMENT_EXTRACT_AUDIO_BASE_WORKERS";
pub(super) const AUDIO_RECOVERY_WORKERS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_RECOVERY_WORKERS";
pub(super) const AUDIO_SPEECH_SEGMENTS_JSONL_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_SEGMENTS_JSONL";
pub(super) const AUDIO_SPEECH_MERGE_GAP_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MERGE_GAP_MS";
pub(super) const AUDIO_SPEECH_MIN_WINDOW_MS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_MIN_WINDOW_MS";
pub(super) const AUDIO_SPEECH_LIMIT_CHUNKS_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_AUDIO_SPEECH_LIMIT_CHUNKS";

const DEFAULT_BACKEND_PROFILE: &str = "hosted-audio-transcript-v1";
const DEFAULT_CHUNK_MS: u64 = 60_000;
const DEFAULT_CONTEXT_MS: u64 = 0;
const DEFAULT_RECOVERY_SPLIT_MS: u64 = 30_000;
const DEFAULT_SPEECH_MERGE_GAP_MS: u64 = 500;
const DEFAULT_SPEECH_MIN_WINDOW_MS: u64 = 0;
const DEFAULT_SPEECH_LIMIT_CHUNKS: u32 = 10_000;
const DEFAULT_SAMPLE_RATE_HZ: u32 = 16_000;
const DEFAULT_CHANNELS: u8 = 1;
const DEFAULT_AUDIO_FORMAT: &str = "wav";
const DEFAULT_FFMPEG: &str = "ffmpeg";
const DEFAULT_FFPROBE: &str = "ffprobe";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AudioDocumentExtractConfig {
    pub(crate) backend_profile: String,
    pub(crate) chunk_duration_ms: u64,
    pub(crate) context_before_ms: u64,
    pub(crate) context_after_ms: u64,
    pub(crate) recovery_split_duration_ms: u64,
    pub(crate) sample_rate_hz: u32,
    pub(crate) channels: u8,
    pub(crate) audio_format: String,
    pub(crate) ffmpeg_path: PathBuf,
    pub(crate) ffprobe_path: PathBuf,
    pub(crate) base_worker_budget: Option<usize>,
    pub(crate) recovery_worker_budget: Option<usize>,
    pub(crate) speech_segments_jsonl_path: Option<PathBuf>,
    pub(crate) speech_merge_gap_ms: u64,
    pub(crate) speech_min_window_ms: u64,
    pub(crate) speech_limit_chunks: u32,
}

impl AudioDocumentExtractConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        document_extract_audio_config(&|key| std::env::var(key).ok())
    }
}

pub(crate) fn document_extract_audio_config(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<AudioDocumentExtractConfig, String> {
    let backend_profile = string_value(lookup, AUDIO_BACKEND_PROFILE_ENV, DEFAULT_BACKEND_PROFILE);
    if backend_profile.trim().is_empty() {
        return Err(format!("{AUDIO_BACKEND_PROFILE_ENV} must not be blank"));
    }
    Ok(AudioDocumentExtractConfig {
        backend_profile,
        chunk_duration_ms: u64_value(lookup, AUDIO_CHUNK_MS_ENV, DEFAULT_CHUNK_MS)?,
        context_before_ms: u64_value(lookup, AUDIO_CONTEXT_BEFORE_MS_ENV, DEFAULT_CONTEXT_MS)?,
        context_after_ms: u64_value(lookup, AUDIO_CONTEXT_AFTER_MS_ENV, DEFAULT_CONTEXT_MS)?,
        recovery_split_duration_ms: u64_value(
            lookup,
            AUDIO_RECOVERY_SPLIT_MS_ENV,
            DEFAULT_RECOVERY_SPLIT_MS,
        )?,
        sample_rate_hz: u32_value(lookup, AUDIO_SAMPLE_RATE_HZ_ENV, DEFAULT_SAMPLE_RATE_HZ)?,
        channels: u8_value(lookup, AUDIO_CHANNELS_ENV, DEFAULT_CHANNELS)?,
        audio_format: string_value(lookup, AUDIO_FORMAT_ENV, DEFAULT_AUDIO_FORMAT),
        ffmpeg_path: PathBuf::from(string_value(lookup, AUDIO_FFMPEG_ENV, DEFAULT_FFMPEG)),
        ffprobe_path: PathBuf::from(string_value(lookup, AUDIO_FFPROBE_ENV, DEFAULT_FFPROBE)),
        base_worker_budget: audio_worker_budget_with_lookup(lookup, AUDIO_BASE_WORKERS_ENV)?,
        recovery_worker_budget: audio_worker_budget_with_lookup(
            lookup,
            AUDIO_RECOVERY_WORKERS_ENV,
        )?,
        speech_segments_jsonl_path: optional_path_value(lookup, AUDIO_SPEECH_SEGMENTS_JSONL_ENV),
        speech_merge_gap_ms: u64_value(
            lookup,
            AUDIO_SPEECH_MERGE_GAP_MS_ENV,
            DEFAULT_SPEECH_MERGE_GAP_MS,
        )?,
        speech_min_window_ms: u64_value(
            lookup,
            AUDIO_SPEECH_MIN_WINDOW_MS_ENV,
            DEFAULT_SPEECH_MIN_WINDOW_MS,
        )?,
        speech_limit_chunks: u32_value(
            lookup,
            AUDIO_SPEECH_LIMIT_CHUNKS_ENV,
            DEFAULT_SPEECH_LIMIT_CHUNKS,
        )?,
    })
}

pub(crate) fn audio_worker_budget_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
) -> Result<Option<usize>, String> {
    let Some(value) = lookup(key) else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{key} must be `auto` or a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{key} must be greater than zero"));
    }
    Ok(Some(parsed))
}

fn string_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: &'static str,
) -> String {
    lookup(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

fn optional_path_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
) -> Option<PathBuf> {
    lookup(key)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn u64_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: u64,
) -> Result<u64, String> {
    parse_positive_u64(lookup(key).as_deref(), key, default)
}

fn u32_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: u32,
) -> Result<u32, String> {
    let value = parse_positive_u64(lookup(key).as_deref(), key, u64::from(default))?;
    u32::try_from(value).map_err(|_| format!("{key} exceeds u32::MAX"))
}

fn u8_value(
    lookup: &dyn Fn(&str) -> Option<String>,
    key: &'static str,
    default: u8,
) -> Result<u8, String> {
    let value = parse_positive_u64(lookup(key).as_deref(), key, u64::from(default))?;
    u8::try_from(value).map_err(|_| format!("{key} exceeds u8::MAX"))
}

fn parse_positive_u64(value: Option<&str>, key: &'static str, default: u64) -> Result<u64, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(default);
    };
    let parsed = value
        .parse::<u64>()
        .map_err(|_| format!("{key} must be a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{key} must be greater than zero"));
    }
    Ok(parsed)
}
