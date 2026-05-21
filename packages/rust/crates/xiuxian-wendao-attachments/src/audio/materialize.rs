//! Parallel media materialization for planned `audio` shards.

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::Digest;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::process::Command;

use super::plan::plan_audio_shards;
use super::types::{
    AudioShardManifestItem, AudioShardMaterializationInput, AudioShardMaterializedItem,
    AudioShardPlan,
};

/// Materialize planned audio shards with local `ffmpeg` in parallel.
///
/// # Errors
///
/// Returns an error when the output directory cannot be created, shard planning
/// fails, a shard media window is empty, or `ffmpeg` exits unsuccessfully.
pub fn materialize_audio_shards(
    plan: &AudioShardPlan,
    input: &AudioShardMaterializationInput,
) -> Result<Vec<AudioShardMaterializedItem>, String> {
    fs::create_dir_all(input.output_dir.as_path()).map_err(|error| {
        format!(
            "failed to create audio shard output dir {}: {error}",
            input.output_dir.display()
        )
    })?;
    let manifests = plan_audio_shards(plan)?;
    manifests
        .into_par_iter()
        .map(|manifest| materialize_one(plan, input, manifest))
        .collect()
}

fn materialize_one(
    plan: &AudioShardPlan,
    input: &AudioShardMaterializationInput,
    manifest: AudioShardManifestItem,
) -> Result<AudioShardMaterializedItem, String> {
    if manifest.media_duration_ms == 0 {
        return Err(format!(
            "audio shard {} has empty media duration",
            manifest.shard_id
        ));
    }
    let output_path = input.output_dir.join(shard_file_name(&manifest));
    if output_path.exists() && !input.force {
        let shard_sha256 = file_sha256_hex(output_path.as_path())?;
        return Ok(AudioShardMaterializedItem {
            manifest,
            output_path,
            shard_sha256,
        });
    }
    let status = Command::new(input.ffmpeg_path.as_path())
        .args(ffmpeg_args(
            plan,
            &manifest,
            &input.source_path,
            &output_path,
        ))
        .status()
        .map_err(|error| {
            format!(
                "failed to launch audio splitter {}: {error}",
                input.ffmpeg_path.display()
            )
        })?;
    if !status.success() {
        return Err(format!(
            "audio splitter failed for shard {} with status {status}",
            manifest.shard_id
        ));
    }
    Ok(AudioShardMaterializedItem {
        shard_sha256: file_sha256_hex(output_path.as_path())?,
        manifest,
        output_path,
    })
}

fn shard_file_name(manifest: &AudioShardManifestItem) -> String {
    let shard_prefix: String = manifest.shard_id.chars().take(16).collect();
    format!(
        "audio_{:06}_{}.{}",
        manifest.chunk_index, shard_prefix, manifest.audio_format
    )
}

fn ffmpeg_args(
    plan: &AudioShardPlan,
    manifest: &AudioShardManifestItem,
    source_path: &std::path::Path,
    output_path: &std::path::Path,
) -> Vec<OsString> {
    [
        "-hide_banner".into(),
        "-nostdin".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-ss".into(),
        seconds_arg(manifest.media_start_ms),
        "-t".into(),
        seconds_arg(manifest.media_duration_ms),
        "-i".into(),
        source_path.as_os_str().to_owned(),
        "-ac".into(),
        plan.channels.to_string().into(),
        "-ar".into(),
        plan.sample_rate_hz.to_string().into(),
        "-vn".into(),
        output_path.as_os_str().to_owned(),
    ]
    .into()
}

fn seconds_arg(ms: u64) -> OsString {
    let seconds = ms / 1000;
    let milliseconds = ms % 1000;
    format!("{seconds}.{milliseconds:03}").into()
}

fn file_sha256_hex(path: &std::path::Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("failed to open audio shard {}: {error}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|error| format!("failed to read audio shard {}: {error}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
