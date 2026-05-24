//! Parallel media materialization for planned `audio` shards.

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::Digest;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::process::Command;
#[cfg(feature = "artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobWrite, ArtifactKey, ArtifactKeyParts, ArtifactKind,
    ContentAddressedFilesystemBlobCache,
};

#[cfg(feature = "artifact-cache")]
use super::identity::sha256_hex;
use super::plan::plan_audio_shards;
use super::types::{
    AudioShardManifestItem, AudioShardMaterializationInput, AudioShardMaterializationSource,
    AudioShardMaterializedItem, AudioShardPlan,
};

#[cfg(feature = "artifact-cache")]
const AUDIO_SHARD_ARTIFACT_CACHE_NAMESPACE: &str = "audio-shards";

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
            materialization_source: AudioShardMaterializationSource::ExistingOutput,
        });
    }
    #[cfg(feature = "artifact-cache")]
    if let Some(shard_sha256) =
        restore_audio_shard_from_artifact_cache(input, &manifest, output_path.as_path())?
    {
        return Ok(AudioShardMaterializedItem {
            manifest,
            output_path,
            shard_sha256,
            materialization_source: AudioShardMaterializationSource::ArtifactCache,
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
    #[cfg(feature = "artifact-cache")]
    write_audio_shard_to_artifact_cache(input, &manifest, output_path.as_path())?;
    Ok(AudioShardMaterializedItem {
        shard_sha256: file_sha256_hex(output_path.as_path())?,
        manifest,
        output_path,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    })
}

#[cfg(feature = "artifact-cache")]
fn restore_audio_shard_from_artifact_cache(
    input: &AudioShardMaterializationInput,
    manifest: &AudioShardManifestItem,
    output_path: &std::path::Path,
) -> Result<Option<String>, String> {
    let Some(cache_root) = input.artifact_cache_dir.as_ref() else {
        return Ok(None);
    };
    let cache = ContentAddressedFilesystemBlobCache::new(cache_root.clone());
    let key = audio_shard_artifact_key(manifest)?;
    let Some(read) = cache
        .read(&key)
        .map_err(|error| format!("read audio shard artifact cache: {error}"))?
    else {
        return Ok(None);
    };
    fs::write(output_path, read.bytes()).map_err(|error| {
        format!(
            "failed to restore audio shard {} from artifact cache: {error}",
            output_path.display()
        )
    })?;
    file_sha256_hex(output_path).map(Some)
}

#[cfg(feature = "artifact-cache")]
fn write_audio_shard_to_artifact_cache(
    input: &AudioShardMaterializationInput,
    manifest: &AudioShardManifestItem,
    output_path: &std::path::Path,
) -> Result<(), String> {
    let Some(cache_root) = input.artifact_cache_dir.as_ref() else {
        return Ok(());
    };
    let bytes = fs::read(output_path).map_err(|error| {
        format!(
            "failed to read audio shard {} for artifact cache: {error}",
            output_path.display()
        )
    })?;
    let cache = ContentAddressedFilesystemBlobCache::new(cache_root.clone());
    let key = audio_shard_artifact_key(manifest)?;
    cache
        .write(&key, ArtifactBlobWrite::new(bytes.as_slice()))
        .map_err(|error| format!("write audio shard artifact cache: {error}"))?;
    Ok(())
}

#[cfg(feature = "artifact-cache")]
fn audio_shard_artifact_key(manifest: &AudioShardManifestItem) -> Result<ArtifactKey, String> {
    ArtifactKey::from_parts(ArtifactKeyParts {
        namespace: AUDIO_SHARD_ARTIFACT_CACHE_NAMESPACE.to_owned(),
        kind: ArtifactKind::AudioChunk,
        source_digest: manifest.source_sha256.clone(),
        profile_digest: sha256_hex(manifest.cache_key.as_bytes()),
        shard_digest: manifest.shard_id.clone(),
    })
    .map_err(|error| format!("build audio shard artifact cache key: {error}"))
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
