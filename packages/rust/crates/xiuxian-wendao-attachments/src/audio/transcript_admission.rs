//! Rust-owned accepted transcript admission for audio shard execution.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::{
    AudioShardInput, AudioShardManifestItem, AudioShardResult, AudioShardResultStatus,
    AudioShardWorkerProfile, merge_audio_shard_results,
};

const AUDIO_TRANSCRIPT_ADMISSION_SCHEMA: &str = "xiuxian_wendao.audio_transcript_result_cache.v1";
const AUDIO_PLANNED_TRANSCRIPT_ADMISSION_SCHEMA: &str =
    "xiuxian_wendao.audio_planned_transcript_result_cache.v1";

/// Runtime counters for one audio transcript admission lookup/persist pass.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTranscriptAdmissionStats {
    /// Whether a transcript admission root was configured.
    pub enabled: bool,
    /// Input rows satisfied from accepted transcript admission.
    pub hit_count: usize,
    /// Input rows that needed analyzer execution.
    pub miss_count: usize,
    /// Successful analyzer rows persisted to accepted transcript admission.
    pub stored_count: usize,
    /// Admission records that existed but failed identity or transcript validation.
    pub stale_count: usize,
    /// Planned shard rows satisfied before byte materialization.
    #[serde(default)]
    pub planned_hit_count: usize,
    /// Planned shard rows missing before byte materialization.
    #[serde(default)]
    pub planned_miss_count: usize,
    /// Planned shard admission records that existed but failed validation.
    #[serde(default)]
    pub planned_stale_count: usize,
    /// Accepted analyzer rows persisted to the planned result index.
    #[serde(default)]
    pub planned_stored_count: usize,
}

impl AudioTranscriptAdmissionStats {
    pub fn add_assign(&mut self, other: &Self) {
        self.enabled |= other.enabled;
        self.hit_count += other.hit_count;
        self.miss_count += other.miss_count;
        self.stored_count += other.stored_count;
        self.stale_count += other.stale_count;
        self.planned_hit_count += other.planned_hit_count;
        self.planned_miss_count += other.planned_miss_count;
        self.planned_stale_count += other.planned_stale_count;
        self.planned_stored_count += other.planned_stored_count;
    }
}

#[derive(Debug, Clone)]
pub struct AudioTranscriptAdmissionLookup {
    pub admitted_results: HashMap<String, AudioShardResult>,
    pub miss_inputs: Vec<AudioShardInput>,
    pub stats: AudioTranscriptAdmissionStats,
}

#[derive(Debug, Clone)]
pub struct AudioPlannedTranscriptAdmissionLookup {
    pub all_hit: bool,
    pub inputs: Vec<AudioShardInput>,
    pub results: Vec<AudioShardResult>,
    pub stats: AudioTranscriptAdmissionStats,
}

/// Backend and storage identity inputs for accepted audio transcript admission.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTranscriptAdmissionOptions {
    /// Optional analyzer audio worker selector.
    pub audio_worker: Option<String>,
    /// Optional hosted audio provider override.
    pub hosted_provider: Option<String>,
    /// Optional hosted audio base URL override.
    pub hosted_base_url: Option<String>,
    /// Optional hosted audio endpoint-kind override.
    pub hosted_endpoint: Option<String>,
    /// Optional hosted audio model override.
    pub hosted_model: Option<String>,
    /// Optional Rust-owned transcript admission root.
    pub admission_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioTranscriptAdmissionRecord {
    schema: String,
    cache_key: String,
    result: AudioShardResult,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AudioPlannedTranscriptAdmissionRecord {
    schema: String,
    cache_key: String,
    input: AudioShardInput,
    result: AudioShardResult,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioTranscriptAdmissionKey<'a> {
    schema: &'static str,
    source_content_hash: &'a str,
    shard_sha256: &'a str,
    shard_profile: &'a str,
    task_profile: &'a str,
    backend_profile: &'a str,
    shard_element_id: &'a str,
    audio_worker: Option<&'a str>,
    hosted_provider: Option<&'a str>,
    hosted_base_url: Option<&'a str>,
    hosted_endpoint: Option<&'a str>,
    hosted_model: Option<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioPlannedTranscriptAdmissionKey<'a> {
    schema: &'static str,
    source_content_hash: &'a str,
    source_path: &'a str,
    shard_profile: &'a str,
    task_profile: &'a str,
    backend_profile: &'a str,
    shard_element_id: &'a str,
    reading_order_key: &'a str,
    sample_rate_hz: u32,
    channels: u8,
    audio_format: &'a str,
    start_ms: u64,
    duration_ms: u64,
    media_start_ms: u64,
    media_duration_ms: u64,
    context_before_ms: u64,
    context_after_ms: u64,
    audio_worker: Option<&'a str>,
    hosted_provider: Option<&'a str>,
    hosted_base_url: Option<&'a str>,
    hosted_endpoint: Option<&'a str>,
    hosted_model: Option<&'a str>,
}

enum AudioTranscriptAdmissionLookupRow {
    Hit(String, AudioShardResult),
    Miss(AudioShardInput),
    Stale(AudioShardInput),
}

enum AudioPlannedTranscriptAdmissionLookupRow {
    Hit(AudioShardInput, AudioShardResult),
    Miss,
    Stale,
}

/// Look up accepted planned audio shard results before byte materialization.
///
/// # Errors
///
/// Returns an error when admission identity construction fails.
pub fn lookup_planned_audio_transcript_admission(
    manifests: &[AudioShardManifestItem],
    profile: &AudioShardWorkerProfile,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<AudioPlannedTranscriptAdmissionLookup, String> {
    let Some(cache_dir) = request_options.admission_dir.as_deref() else {
        return Ok(AudioPlannedTranscriptAdmissionLookup {
            all_hit: false,
            inputs: Vec::new(),
            results: Vec::new(),
            stats: AudioTranscriptAdmissionStats::default(),
        });
    };
    let rows = manifests
        .iter()
        .map(|manifest| {
            lookup_planned_audio_transcript_admission_row(
                cache_dir,
                manifest,
                profile,
                request_options,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let hit_count = rows
        .iter()
        .filter(|row| matches!(row, AudioPlannedTranscriptAdmissionLookupRow::Hit(_, _)))
        .count();
    let miss_count = rows
        .iter()
        .filter(|row| matches!(row, AudioPlannedTranscriptAdmissionLookupRow::Miss))
        .count();
    let stale_count = rows
        .iter()
        .filter(|row| matches!(row, AudioPlannedTranscriptAdmissionLookupRow::Stale))
        .count();
    let all_hit = !manifests.is_empty() && hit_count == manifests.len();
    let (inputs, results) = if all_hit {
        rows.into_iter()
            .filter_map(|row| match row {
                AudioPlannedTranscriptAdmissionLookupRow::Hit(input, result) => {
                    Some((input, result))
                }
                AudioPlannedTranscriptAdmissionLookupRow::Miss
                | AudioPlannedTranscriptAdmissionLookupRow::Stale => None,
            })
            .unzip()
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(AudioPlannedTranscriptAdmissionLookup {
        all_hit,
        inputs,
        results,
        stats: AudioTranscriptAdmissionStats {
            enabled: true,
            hit_count: if all_hit { hit_count } else { 0 },
            planned_hit_count: hit_count,
            planned_miss_count: miss_count,
            planned_stale_count: stale_count,
            ..AudioTranscriptAdmissionStats::default()
        },
    })
}

/// Look up accepted audio shard result rows for already materialized inputs.
///
/// # Errors
///
/// Returns an error when admission identity construction fails.
pub fn lookup_audio_transcript_admission(
    inputs: &[AudioShardInput],
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<AudioTranscriptAdmissionLookup, String> {
    let Some(cache_dir) = request_options.admission_dir.as_deref() else {
        return Ok(AudioTranscriptAdmissionLookup {
            admitted_results: HashMap::new(),
            miss_inputs: inputs.to_vec(),
            stats: AudioTranscriptAdmissionStats {
                enabled: false,
                miss_count: inputs.len(),
                ..AudioTranscriptAdmissionStats::default()
            },
        });
    };
    let rows = inputs
        .iter()
        .map(|input| lookup_audio_transcript_admission_row(cache_dir, input, request_options))
        .collect::<Result<Vec<_>, _>>()?;
    let admitted_results = rows
        .iter()
        .filter_map(|row| match row {
            AudioTranscriptAdmissionLookupRow::Hit(shard_id, result) => {
                Some((shard_id.clone(), result.clone()))
            }
            AudioTranscriptAdmissionLookupRow::Miss(_)
            | AudioTranscriptAdmissionLookupRow::Stale(_) => None,
        })
        .collect::<HashMap<_, _>>();
    let miss_inputs = rows
        .iter()
        .filter_map(|row| match row {
            AudioTranscriptAdmissionLookupRow::Miss(input)
            | AudioTranscriptAdmissionLookupRow::Stale(input) => Some(input.clone()),
            AudioTranscriptAdmissionLookupRow::Hit(_, _) => None,
        })
        .collect::<Vec<_>>();
    let planned_backfill_count = backfill_planned_transcript_admission(
        cache_dir,
        inputs,
        &admitted_results,
        request_options,
    )?;
    let stats = AudioTranscriptAdmissionStats {
        enabled: true,
        hit_count: admitted_results.len(),
        miss_count: miss_inputs.len(),
        stale_count: rows
            .iter()
            .filter(|row| matches!(row, AudioTranscriptAdmissionLookupRow::Stale(_)))
            .count(),
        planned_stored_count: planned_backfill_count,
        ..AudioTranscriptAdmissionStats::default()
    };
    Ok(AudioTranscriptAdmissionLookup {
        admitted_results,
        miss_inputs,
        stats,
    })
}

/// Persist accepted audio shard result rows to normal and planned admissions.
///
/// # Errors
///
/// Returns an error when admission identity construction, serialization, or file
/// writes fail.
pub fn persist_audio_transcript_admission(
    inputs: &[AudioShardInput],
    results: &[AudioShardResult],
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<AudioTranscriptAdmissionStats, String> {
    let Some(cache_dir) = request_options.admission_dir.as_deref() else {
        return Ok(AudioTranscriptAdmissionStats::default());
    };
    let mut stats = AudioTranscriptAdmissionStats {
        enabled: true,
        ..AudioTranscriptAdmissionStats::default()
    };
    let results_by_shard = audio_results_by_shard(results);
    for input in inputs {
        let Some(candidates) = results_by_shard.get(input.shard_element_id.as_str()) else {
            continue;
        };
        if candidates.len() != 1 {
            continue;
        }
        let result = candidates[0];
        if !is_admissible_transcript_result(input, result) {
            continue;
        }
        let cache_key = audio_transcript_admission_key(input, request_options)?;
        let cache_path = audio_transcript_admission_path(cache_dir, cache_key.as_str());
        let record = AudioTranscriptAdmissionRecord {
            schema: AUDIO_TRANSCRIPT_ADMISSION_SCHEMA.to_owned(),
            cache_key,
            result: result.clone(),
        };
        write_audio_transcript_admission_record(cache_path.as_path(), &record)?;
        write_planned_audio_transcript_admission_record_for_input(
            cache_dir,
            input,
            result,
            request_options,
        )?;
        stats.stored_count += 1;
        stats.planned_stored_count += 1;
    }
    Ok(stats)
}

/// Combine admitted and fresh transcript rows in input order.
#[must_use]
pub fn combine_admitted_and_fresh_audio_transcripts(
    inputs: &[AudioShardInput],
    admitted_results: &HashMap<String, AudioShardResult>,
    fresh_results: &[AudioShardResult],
) -> Vec<AudioShardResult> {
    let fresh_by_shard = audio_results_by_shard(fresh_results);
    let input_shards = inputs
        .iter()
        .map(|input| input.shard_element_id.as_str())
        .collect::<HashSet<_>>();
    let consumed_fresh_shards = inputs
        .iter()
        .filter(|input| !admitted_results.contains_key(input.shard_element_id.as_str()))
        .filter(|input| fresh_by_shard.contains_key(input.shard_element_id.as_str()))
        .map(|input| input.shard_element_id.as_str())
        .collect::<HashSet<_>>();
    let ordered_results = inputs.iter().flat_map(|input| {
        if let Some(result) = admitted_results.get(input.shard_element_id.as_str()) {
            return vec![result.clone()];
        }
        fresh_by_shard
            .get(input.shard_element_id.as_str())
            .map(|results| results.iter().map(|result| (*result).clone()).collect())
            .unwrap_or_default()
    });
    let extra_results = fresh_results.iter().filter_map(|result| {
        let shard_id = result.shard_element_id.as_str();
        (!consumed_fresh_shards.contains(shard_id) && !input_shards.contains(shard_id))
            .then(|| result.clone())
    });
    ordered_results.chain(extra_results).collect()
}

fn lookup_audio_transcript_admission_row(
    cache_dir: &Path,
    input: &AudioShardInput,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<AudioTranscriptAdmissionLookupRow, String> {
    let cache_key = audio_transcript_admission_key(input, request_options)?;
    let cache_path = audio_transcript_admission_path(cache_dir, cache_key.as_str());
    match read_audio_transcript_admission_record(cache_path.as_path(), cache_key.as_str(), input) {
        Ok(Some(result)) => Ok(AudioTranscriptAdmissionLookupRow::Hit(
            input.shard_element_id.clone(),
            result,
        )),
        Ok(None) => Ok(AudioTranscriptAdmissionLookupRow::Miss(input.clone())),
        Err(_) => Ok(AudioTranscriptAdmissionLookupRow::Stale(input.clone())),
    }
}

fn lookup_planned_audio_transcript_admission_row(
    cache_dir: &Path,
    manifest: &AudioShardManifestItem,
    profile: &AudioShardWorkerProfile,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<AudioPlannedTranscriptAdmissionLookupRow, String> {
    let cache_key =
        audio_planned_transcript_admission_key_from_manifest(manifest, profile, request_options)?;
    let cache_path = audio_transcript_admission_path(cache_dir, cache_key.as_str());
    match read_planned_audio_transcript_admission_record(
        cache_path.as_path(),
        cache_key.as_str(),
        manifest,
        profile,
    ) {
        Ok(Some((input, result))) => {
            Ok(AudioPlannedTranscriptAdmissionLookupRow::Hit(input, result))
        }
        Ok(None) => Ok(AudioPlannedTranscriptAdmissionLookupRow::Miss),
        Err(_) => Ok(AudioPlannedTranscriptAdmissionLookupRow::Stale),
    }
}

fn read_audio_transcript_admission_record(
    cache_path: &Path,
    expected_cache_key: &str,
    input: &AudioShardInput,
) -> Result<Option<AudioShardResult>, String> {
    if !cache_path.exists() {
        return Ok(None);
    }
    let payload = std::fs::read(cache_path).map_err(|error| {
        format!(
            "failed to read audio transcript admission {}: {error}",
            cache_path.display()
        )
    })?;
    let record = serde_json::from_slice::<AudioTranscriptAdmissionRecord>(payload.as_slice())
        .map_err(|error| {
            format!(
                "failed to decode audio transcript admission {}: {error}",
                cache_path.display()
            )
        })?;
    if record.schema != AUDIO_TRANSCRIPT_ADMISSION_SCHEMA || record.cache_key != expected_cache_key
    {
        return Err("audio transcript admission identity mismatch".to_owned());
    }
    if !is_admissible_transcript_result(input, &record.result) {
        return Err("audio transcript admission validation failed".to_owned());
    }
    Ok(Some(record.result))
}

fn read_planned_audio_transcript_admission_record(
    cache_path: &Path,
    expected_cache_key: &str,
    manifest: &AudioShardManifestItem,
    profile: &AudioShardWorkerProfile,
) -> Result<Option<(AudioShardInput, AudioShardResult)>, String> {
    if !cache_path.exists() {
        return Ok(None);
    }
    let payload = std::fs::read(cache_path).map_err(|error| {
        format!(
            "failed to read audio planned transcript admission {}: {error}",
            cache_path.display()
        )
    })?;
    let record =
        serde_json::from_slice::<AudioPlannedTranscriptAdmissionRecord>(payload.as_slice())
            .map_err(|error| {
                format!(
                    "failed to decode audio planned transcript admission {}: {error}",
                    cache_path.display()
                )
            })?;
    if record.schema != AUDIO_PLANNED_TRANSCRIPT_ADMISSION_SCHEMA
        || record.cache_key != expected_cache_key
    {
        return Err("audio planned transcript admission identity mismatch".to_owned());
    }
    if !planned_input_matches_manifest(&record.input, manifest, profile)
        || !is_admissible_transcript_result(&record.input, &record.result)
    {
        return Err("audio planned transcript admission validation failed".to_owned());
    }
    Ok(Some((record.input, record.result)))
}

fn write_audio_transcript_admission_record(
    cache_path: &Path,
    record: &AudioTranscriptAdmissionRecord,
) -> Result<(), String> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create audio transcript admission dir {}: {error}",
                parent.display()
            )
        })?;
    }
    let payload = serde_json::to_vec_pretty(record)
        .map_err(|error| format!("serialize audio transcript admission: {error}"))?;
    std::fs::write(cache_path, payload).map_err(|error| {
        format!(
            "failed to write audio transcript admission {}: {error}",
            cache_path.display()
        )
    })
}

fn write_planned_audio_transcript_admission_record(
    cache_path: &Path,
    record: &AudioPlannedTranscriptAdmissionRecord,
) -> Result<(), String> {
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create audio planned transcript admission dir {}: {error}",
                parent.display()
            )
        })?;
    }
    let payload = serde_json::to_vec_pretty(record)
        .map_err(|error| format!("serialize audio planned transcript admission: {error}"))?;
    std::fs::write(cache_path, payload).map_err(|error| {
        format!(
            "failed to write audio planned transcript admission {}: {error}",
            cache_path.display()
        )
    })
}

fn backfill_planned_transcript_admission(
    cache_dir: &Path,
    inputs: &[AudioShardInput],
    admitted_results: &HashMap<String, AudioShardResult>,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<usize, String> {
    let mut stored_count = 0;
    for input in inputs {
        let Some(result) = admitted_results.get(input.shard_element_id.as_str()) else {
            continue;
        };
        write_planned_audio_transcript_admission_record_for_input(
            cache_dir,
            input,
            result,
            request_options,
        )?;
        stored_count += 1;
    }
    Ok(stored_count)
}

fn write_planned_audio_transcript_admission_record_for_input(
    cache_dir: &Path,
    input: &AudioShardInput,
    result: &AudioShardResult,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<(), String> {
    let planned_cache_key =
        audio_planned_transcript_admission_key_from_input(input, request_options)?;
    let planned_cache_path = audio_transcript_admission_path(cache_dir, planned_cache_key.as_str());
    let planned_record = AudioPlannedTranscriptAdmissionRecord {
        schema: AUDIO_PLANNED_TRANSCRIPT_ADMISSION_SCHEMA.to_owned(),
        cache_key: planned_cache_key,
        input: input.clone(),
        result: result.clone(),
    };
    write_planned_audio_transcript_admission_record(planned_cache_path.as_path(), &planned_record)
}

fn is_admissible_transcript_result(input: &AudioShardInput, result: &AudioShardResult) -> bool {
    if result.status != AudioShardResultStatus::Succeeded {
        return false;
    }
    let Some(text) = result.text.as_deref() else {
        return false;
    };
    if text.trim().is_empty() {
        return false;
    }
    let Ok(report) =
        merge_audio_shard_results(std::slice::from_ref(input), std::slice::from_ref(result))
    else {
        return false;
    };
    report.has_complete_success_coverage()
}

fn audio_results_by_shard(results: &[AudioShardResult]) -> HashMap<&str, Vec<&AudioShardResult>> {
    let mut results_by_shard = HashMap::<&str, Vec<&AudioShardResult>>::new();
    for result in results {
        results_by_shard
            .entry(result.shard_element_id.as_str())
            .or_default()
            .push(result);
    }
    results_by_shard
}

fn planned_input_matches_manifest(
    input: &AudioShardInput,
    manifest: &AudioShardManifestItem,
    profile: &AudioShardWorkerProfile,
) -> bool {
    input.source_path == manifest.source_id
        && input.source_content_hash == manifest.source_sha256
        && input.shard_profile == manifest_shard_profile(manifest)
        && input.task_profile == profile.task_profile
        && input.backend_profile == profile.backend_profile
        && input.preferred_languages == profile.preferred_languages
        && input.sample_rate_hz == manifest.sample_rate_hz
        && input.channels == manifest.channels
        && input.audio_format == manifest.audio_format
        && input.start_ms == manifest.start_ms
        && input.duration_ms == manifest.duration_ms
        && input.media_start_ms == manifest.media_start_ms
        && input.media_duration_ms == manifest.media_duration_ms
        && input.context_before_ms == manifest.context_before_ms
        && input.context_after_ms == manifest.context_after_ms
        && input.shard_element_id == manifest.shard_id
        && input.reading_order_key == manifest.reading_order_key
}

fn manifest_shard_profile(manifest: &AudioShardManifestItem) -> &str {
    manifest.cache_key.split(':').next().unwrap_or("")
}

pub fn audio_transcript_admission_key(
    input: &AudioShardInput,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<String, String> {
    let key = AudioTranscriptAdmissionKey {
        schema: AUDIO_TRANSCRIPT_ADMISSION_SCHEMA,
        source_content_hash: input.source_content_hash.as_str(),
        shard_sha256: input.shard_sha256.as_str(),
        shard_profile: input.shard_profile.as_str(),
        task_profile: input.task_profile.as_str(),
        backend_profile: input.backend_profile.as_str(),
        shard_element_id: input.shard_element_id.as_str(),
        audio_worker: request_options.audio_worker.as_deref(),
        hosted_provider: request_options.hosted_provider.as_deref(),
        hosted_base_url: request_options.hosted_base_url.as_deref(),
        hosted_endpoint: request_options.hosted_endpoint.as_deref(),
        hosted_model: request_options.hosted_model.as_deref(),
    };
    let payload = serde_json::to_vec(&key)
        .map_err(|error| format!("serialize audio transcript admission key: {error}"))?;
    Ok(format!("{:x}", sha2::Sha256::digest(payload.as_slice())))
}

fn audio_planned_transcript_admission_key_from_input(
    input: &AudioShardInput,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<String, String> {
    let key = AudioPlannedTranscriptAdmissionKey {
        schema: AUDIO_PLANNED_TRANSCRIPT_ADMISSION_SCHEMA,
        source_content_hash: input.source_content_hash.as_str(),
        source_path: input.source_path.as_str(),
        shard_profile: input.shard_profile.as_str(),
        task_profile: input.task_profile.as_str(),
        backend_profile: input.backend_profile.as_str(),
        shard_element_id: input.shard_element_id.as_str(),
        reading_order_key: input.reading_order_key.as_str(),
        sample_rate_hz: input.sample_rate_hz,
        channels: input.channels,
        audio_format: input.audio_format.as_str(),
        start_ms: input.start_ms,
        duration_ms: input.duration_ms,
        media_start_ms: input.media_start_ms,
        media_duration_ms: input.media_duration_ms,
        context_before_ms: input.context_before_ms,
        context_after_ms: input.context_after_ms,
        audio_worker: request_options.audio_worker.as_deref(),
        hosted_provider: request_options.hosted_provider.as_deref(),
        hosted_base_url: request_options.hosted_base_url.as_deref(),
        hosted_endpoint: request_options.hosted_endpoint.as_deref(),
        hosted_model: request_options.hosted_model.as_deref(),
    };
    audio_planned_transcript_admission_key(key)
}

fn audio_planned_transcript_admission_key_from_manifest(
    manifest: &AudioShardManifestItem,
    profile: &AudioShardWorkerProfile,
    request_options: &AudioTranscriptAdmissionOptions,
) -> Result<String, String> {
    let key = AudioPlannedTranscriptAdmissionKey {
        schema: AUDIO_PLANNED_TRANSCRIPT_ADMISSION_SCHEMA,
        source_content_hash: manifest.source_sha256.as_str(),
        source_path: manifest.source_id.as_str(),
        shard_profile: manifest_shard_profile(manifest),
        task_profile: profile.task_profile.as_str(),
        backend_profile: profile.backend_profile.as_str(),
        shard_element_id: manifest.shard_id.as_str(),
        reading_order_key: manifest.reading_order_key.as_str(),
        sample_rate_hz: manifest.sample_rate_hz,
        channels: manifest.channels,
        audio_format: manifest.audio_format.as_str(),
        start_ms: manifest.start_ms,
        duration_ms: manifest.duration_ms,
        media_start_ms: manifest.media_start_ms,
        media_duration_ms: manifest.media_duration_ms,
        context_before_ms: manifest.context_before_ms,
        context_after_ms: manifest.context_after_ms,
        audio_worker: request_options.audio_worker.as_deref(),
        hosted_provider: request_options.hosted_provider.as_deref(),
        hosted_base_url: request_options.hosted_base_url.as_deref(),
        hosted_endpoint: request_options.hosted_endpoint.as_deref(),
        hosted_model: request_options.hosted_model.as_deref(),
    };
    audio_planned_transcript_admission_key(key)
}

fn audio_planned_transcript_admission_key(
    key: AudioPlannedTranscriptAdmissionKey<'_>,
) -> Result<String, String> {
    let payload = serde_json::to_vec(&key)
        .map_err(|error| format!("serialize audio planned transcript admission key: {error}"))?;
    Ok(format!("{:x}", sha2::Sha256::digest(payload.as_slice())))
}

pub fn audio_transcript_admission_path(cache_dir: &Path, cache_key: &str) -> PathBuf {
    let prefix = cache_key.get(..2).unwrap_or("00");
    cache_dir.join(prefix).join(format!("{cache_key}.json"))
}
