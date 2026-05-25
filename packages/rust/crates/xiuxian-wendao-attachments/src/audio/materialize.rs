//! Parallel media materialization for planned `audio` shards.

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::Digest;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::process::Command;
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobCacheBackend, ArtifactBlobCacheBackendConfig, ArtifactBlobWrite,
    ArtifactKey, ArtifactKeyParts, ArtifactKind,
};

#[cfg(feature = "foyer-artifact-cache")]
use super::identity::sha256_hex;
use super::plan::plan_audio_shards;
use super::types::{
    AudioShardManifestItem, AudioShardMaterializationInput, AudioShardMaterializationSource,
    AudioShardMaterializedItem, AudioShardPlan,
};

#[cfg(feature = "foyer-artifact-cache")]
const AUDIO_SHARD_ARTIFACT_CACHE_NAMESPACE: &str = "audio-shards";

#[derive(Debug, Clone, PartialEq, Eq)]
struct AudioShardDigest {
    sha256: String,
    byte_len: u64,
}

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
    let manifests = plan_audio_shards(plan)?;
    materialize_audio_shard_manifests(plan, input, manifests.as_slice())
}

/// Materialize selected audio shard manifest rows while preserving shard ids.
///
/// # Errors
///
/// Returns an error when the output directory cannot be created, a selected
/// shard media window is empty, or `ffmpeg` exits unsuccessfully.
pub fn materialize_audio_shard_manifests(
    plan: &AudioShardPlan,
    input: &AudioShardMaterializationInput,
    manifests: &[AudioShardManifestItem],
) -> Result<Vec<AudioShardMaterializedItem>, String> {
    fs::create_dir_all(input.output_dir.as_path()).map_err(|error| {
        format!(
            "failed to create audio shard output dir {}: {error}",
            input.output_dir.display()
        )
    })?;
    #[cfg(feature = "foyer-artifact-cache")]
    let artifact_cache = audio_artifact_blob_cache(input)?;
    manifests
        .to_vec()
        .into_par_iter()
        .map(|manifest| {
            #[cfg(feature = "foyer-artifact-cache")]
            {
                materialize_one(plan, input, manifest, artifact_cache.as_ref())
            }
            #[cfg(not(feature = "foyer-artifact-cache"))]
            {
                materialize_one(plan, input, manifest)
            }
        })
        .collect()
}

#[cfg(feature = "foyer-artifact-cache")]
fn materialize_one(
    plan: &AudioShardPlan,
    input: &AudioShardMaterializationInput,
    manifest: AudioShardManifestItem,
    artifact_cache: Option<&ArtifactBlobCacheBackend>,
) -> Result<AudioShardMaterializedItem, String> {
    if manifest.media_duration_ms == 0 {
        return Err(format!(
            "audio shard {} has empty media duration",
            manifest.shard_id
        ));
    }
    let output_path = input.output_dir.join(shard_file_name(&manifest));
    if output_path.exists() && !input.force {
        let digest = file_digest(output_path.as_path())?;
        return Ok(AudioShardMaterializedItem {
            manifest,
            output_path,
            shard_sha256: digest.sha256,
            shard_byte_len: digest.byte_len,
            materialization_source: AudioShardMaterializationSource::ExistingOutput,
        });
    }
    #[cfg(feature = "foyer-artifact-cache")]
    if let Some(cache) = artifact_cache
        && let Some(digest) =
            restore_audio_shard_from_artifact_cache(cache, &manifest, output_path.as_path())?
    {
        return Ok(AudioShardMaterializedItem {
            manifest,
            output_path,
            shard_sha256: digest.sha256,
            shard_byte_len: digest.byte_len,
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
    #[cfg(feature = "foyer-artifact-cache")]
    let (digest, cache_bytes) =
        file_digest_with_optional_bytes(output_path.as_path(), artifact_cache.is_some())?;
    if let Some(cache) = artifact_cache {
        write_audio_shard_to_artifact_cache(cache, &manifest, cache_bytes.as_slice())?;
    }
    Ok(AudioShardMaterializedItem {
        shard_sha256: digest.sha256,
        shard_byte_len: digest.byte_len,
        manifest,
        output_path,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    })
}

#[cfg(not(feature = "foyer-artifact-cache"))]
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
        let digest = file_digest(output_path.as_path())?;
        return Ok(AudioShardMaterializedItem {
            manifest,
            output_path,
            shard_sha256: digest.sha256,
            shard_byte_len: digest.byte_len,
            materialization_source: AudioShardMaterializationSource::ExistingOutput,
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
    let digest = file_digest(output_path.as_path())?;
    Ok(AudioShardMaterializedItem {
        shard_sha256: digest.sha256,
        shard_byte_len: digest.byte_len,
        manifest,
        output_path,
        materialization_source: AudioShardMaterializationSource::MediaSplitter,
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn audio_artifact_blob_cache(
    input: &AudioShardMaterializationInput,
) -> Result<Option<ArtifactBlobCacheBackend>, String> {
    let Some(cache_root) = input.artifact_cache_dir.as_ref() else {
        return Ok(None);
    };
    let config =
        ArtifactBlobCacheBackendConfig::from_root_and_env(cache_root).map_err(|error| {
            format!(
                "resolve audio shard artifact cache backend at `{}`: {error}",
                cache_root.display()
            )
        })?;
    config.build().map(Some).map_err(|error| {
        format!(
            "build audio shard artifact cache backend `{}` at `{}`: {error}",
            config.kind().as_str(),
            config.root().display()
        )
    })
}

#[cfg(feature = "foyer-artifact-cache")]
fn restore_audio_shard_from_artifact_cache(
    cache: &dyn ArtifactBlobCache,
    manifest: &AudioShardManifestItem,
    output_path: &std::path::Path,
) -> Result<Option<AudioShardDigest>, String> {
    let key = audio_shard_artifact_key(manifest)?;
    let Some(read) = cache
        .read(&key)
        .map_err(|error| format!("read audio shard artifact cache: {error}"))?
    else {
        return Ok(None);
    };
    let bytes = read.bytes();
    fs::write(output_path, bytes).map_err(|error| {
        format!(
            "failed to restore audio shard {} from artifact cache: {error}",
            output_path.display()
        )
    })?;
    Ok(Some(bytes_digest(bytes)))
}

#[cfg(feature = "foyer-artifact-cache")]
fn write_audio_shard_to_artifact_cache(
    cache: &dyn ArtifactBlobCache,
    manifest: &AudioShardManifestItem,
    bytes: &[u8],
) -> Result<(), String> {
    let key = audio_shard_artifact_key(manifest)?;
    cache
        .write(&key, ArtifactBlobWrite::new(bytes))
        .map_err(|error| format!("write audio shard artifact cache: {error}"))?;
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
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

fn file_digest(path: &std::path::Path) -> Result<AudioShardDigest, String> {
    file_digest_with_optional_bytes(path, false).map(|(digest, _)| digest)
}

fn file_digest_with_optional_bytes(
    path: &std::path::Path,
    keep_bytes: bool,
) -> Result<(AudioShardDigest, Vec<u8>), String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("failed to open audio shard {}: {error}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut byte_len = 0_u64;
    let mut bytes = Vec::new();
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|error| format!("failed to read audio shard {}: {error}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        let chunk = &buffer[..bytes_read];
        hasher.update(chunk);
        byte_len = byte_len.saturating_add(u64::try_from(bytes_read).unwrap_or(u64::MAX));
        if keep_bytes {
            bytes.extend_from_slice(chunk);
        }
    }
    Ok((
        AudioShardDigest {
            sha256: format!("{:x}", hasher.finalize()),
            byte_len,
        },
        bytes,
    ))
}

#[cfg(feature = "foyer-artifact-cache")]
fn bytes_digest(bytes: &[u8]) -> AudioShardDigest {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    AudioShardDigest {
        sha256: format!("{:x}", hasher.finalize()),
        byte_len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
    }
}
