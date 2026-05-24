//! Source probing and full-timeline audio shard plan construction.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::Digest;
use xiuxian_wendao_attachments::audio::{
    AudioShardMaterializationInput, AudioShardPlan, AudioSourceIdentity,
    DEFAULT_AUDIO_SHARD_PROFILE,
};

use super::config::AudioDocumentExtractConfig;

pub(super) fn probe_audio_duration_ms(source: &Path, ffprobe_path: &Path) -> Result<u64, String> {
    let output = Command::new(ffprobe_path)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(source)
        .output()
        .map_err(|error| {
            format!(
                "failed to launch audio duration probe {}: {error}",
                ffprobe_path.display()
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(output.stderr.as_slice());
        return Err(format!(
            "audio duration probe failed for {} with status {}: {}",
            source.display(),
            output.status,
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    parse_ffprobe_duration_ms(stdout.trim())
}

pub(crate) fn parse_ffprobe_duration_ms(value: &str) -> Result<u64, String> {
    let seconds = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("audio duration probe returned invalid duration `{value}`"))?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(format!(
            "audio duration probe returned non-positive duration `{value}`"
        ));
    }
    let duration = Duration::try_from_secs_f64(seconds)
        .map_err(|_| format!("audio duration probe returned invalid duration `{value}`"))?;
    let base_ms = u64::try_from(duration.as_millis())
        .map_err(|_| format!("audio duration probe returned too large duration `{value}`"))?;
    let has_fractional_ms = duration.subsec_nanos() % 1_000_000 != 0;
    base_ms
        .checked_add(u64::from(has_fractional_ms))
        .ok_or_else(|| format!("audio duration probe returned too large duration `{value}`"))
}

pub(super) fn source_sha256_hex(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("failed to open audio source {}: {error}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|error| format!("failed to read audio source {}: {error}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn build_full_coverage_audio_plan(
    source: &Path,
    source_hash: String,
    duration_ms: u64,
    config: &AudioDocumentExtractConfig,
) -> Result<AudioShardPlan, String> {
    if duration_ms == 0 {
        return Err("audio source duration must be positive".to_owned());
    }
    let mut start_offsets_ms = Vec::new();
    let mut window_durations_ms = Vec::new();
    let mut start_ms = 0_u64;
    while start_ms < duration_ms {
        let remaining_ms = duration_ms.saturating_sub(start_ms);
        let shard_window_ms = remaining_ms.min(config.chunk_duration_ms);
        start_offsets_ms.push(start_ms);
        window_durations_ms.push(shard_window_ms);
        start_ms = start_ms.saturating_add(shard_window_ms);
    }
    Ok(AudioShardPlan {
        profile: DEFAULT_AUDIO_SHARD_PROFILE.to_owned(),
        source: AudioSourceIdentity {
            source_id: source.to_string_lossy().to_string(),
            source_sha256: source_hash,
            duration_ms: Some(duration_ms),
        },
        chunk_duration_ms: config.chunk_duration_ms,
        start_offsets_ms,
        window_durations_ms,
        context_before_ms: config.context_before_ms,
        context_after_ms: config.context_after_ms,
        sample_rate_hz: config.sample_rate_hz,
        channels: config.channels,
        audio_format: config.audio_format.clone(),
        strategy: "full-coverage".to_owned(),
    })
}

pub(super) fn audio_materialization_input(
    source: PathBuf,
    output: &Path,
    config: &AudioDocumentExtractConfig,
    force: bool,
) -> AudioShardMaterializationInput {
    AudioShardMaterializationInput {
        source_path: source,
        output_dir: output.join("audio_shards"),
        ffmpeg_path: config.ffmpeg_path.clone(),
        artifact_cache_dir: config.artifact_cache_dir.clone(),
        force,
    }
}
