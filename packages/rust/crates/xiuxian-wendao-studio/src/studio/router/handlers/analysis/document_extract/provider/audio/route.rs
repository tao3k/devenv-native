//! Studio document-extract route for Rust-planned audio shards.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions,
    AudioShardMaterializationSource, AudioShardMaterializedItem, AudioShardWorkerProfile,
    AudioTranscriptAdmissionStats, apply_audio_recovery_patch_decisions,
};
use xiuxian_wendao_server::transport::{
    DocumentExtractFlightRequest, DocumentExtractFlightRouteResponse,
};

use super::config::AudioDocumentExtractConfig;
use super::plan::{
    audio_materialization_input, build_full_coverage_audio_plan, probe_audio_duration_ms,
    source_sha256_hex,
};
use super::response::build_audio_transcript_with_org_batch;
use super::speech::recovery_speech_window_input_from_config;
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightRequestOptions, AudioShardRecoveryWorkflowRequest,
};
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, mark_document_extract_cache_complete, read_arrow_file,
    write_arrow_file,
};
use crate::studio::router::handlers::analysis::document_extract::provider::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::provider::transport::{
    document_extract_default_endpoint_with_lookup,
    document_extract_endpoint_attempt_order_for_request, document_extract_endpoint_urls,
};
use crate::studio::router::handlers::analysis::document_extract::registry::default_output_dir;

const AUDIO_CACHE_MANIFEST_NAME: &str = "_audio_extract_manifest.json";
const AUDIO_CACHE_MANIFEST_SCHEMA: &str = "xiuxian_wendao.audio_document_extract_cache.v1";
const AUDIO_MATERIALIZATION_REPORT_NAME: &str = "_audio_materialization.json";
const AUDIO_MATERIALIZATION_REPORT_SCHEMA: &str = "xiuxian_wendao.audio_materialization_report.v1";
const AUDIO_TRANSCRIPT_ADMISSION_REPORT_NAME: &str = "_audio_transcript_admission.json";
const AUDIO_TRANSCRIPT_ADMISSION_REPORT_SCHEMA: &str =
    "xiuxian_wendao.audio_transcript_admission_report.v1";

impl StudioDocumentExtractFlightRouteProvider {
    pub(crate) async fn audio_shards_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        self.audio_shards_document_extract_batch_with_config(
            request,
            AudioDocumentExtractConfig::from_env()?,
        )
        .await
    }

    pub(crate) async fn audio_shards_document_extract_batch_with_config(
        &self,
        request: &DocumentExtractFlightRequest,
        config: AudioDocumentExtractConfig,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(request.source_path.as_str());
        let output = document_extract_audio_output_dir(&source, request.output_dir.as_str());
        let duration_ms = probe_audio_duration_ms(source.as_path(), config.ffprobe_path.as_path())?;
        let source_hash = source_sha256_hex(source.as_path())?;
        let cache_manifest =
            audio_cache_manifest(request, &config, source_hash.as_str(), duration_ms);
        if source.exists()
            && !request.force
            && audio_cache_manifest_matches(output.as_path(), &cache_manifest)
            && let Some(batches) = read_cached_audio_document_batches(output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }

        let plan =
            build_full_coverage_audio_plan(source.as_path(), source_hash, duration_ms, &config)?;
        let scheduled_base_worker_budget = config.base_worker_budget.or_else(|| {
            Some(
                self.runtime
                    .audio_capacity
                    .budget_for_shards(plan.start_offsets_ms.len()),
            )
        });
        let scheduled_recovery_worker_budget = config
            .recovery_worker_budget
            .or(scheduled_base_worker_budget);
        let materialization =
            audio_materialization_input(source.clone(), output.as_path(), &config, request.force);
        let recovery_speech_window_input =
            recovery_speech_window_input_from_config(&plan, &config)?;
        let profile = AudioShardWorkerProfile::transcription(config.backend_profile.as_str());
        let client = self.connect_audio_shard_client().await?;
        let _permit = self.acquire_document_extract_dispatch_permit().await?;
        let workflow_started = Instant::now();
        let execution = match client
            .execute_recovery_split_with_options(
                AudioShardRecoveryWorkflowRequest {
                    parent_plan: &plan,
                    materialization: &materialization,
                    profile: &profile,
                    request_metrics: &[],
                    selection_options: AudioRiskParentSelectionOptions::default(),
                    patch_options: AudioRecoveryPatchGateOptions::default(),
                    recovery_split_duration_ms: config.recovery_split_duration_ms,
                    recovery_speech_window_input: recovery_speech_window_input.as_ref(),
                    base_worker_budget: scheduled_base_worker_budget,
                    recovery_worker_budget: scheduled_recovery_worker_budget,
                },
                audio_shard_request_options_for_document_extract(request, &config),
            )
            .await
        {
            Ok(execution) => execution,
            Err(error) => {
                self.runtime.audio_capacity.record_failure();
                return Err(error);
            }
        };
        let output_string = output.to_string_lossy().to_string();
        let final_base_results = apply_audio_recovery_patch_decisions(
            execution.base_response.results.as_slice(),
            &execution.patch_gate_report,
        );
        let batch = match build_audio_transcript_with_org_batch(
            request.source_path.as_str(),
            output_string.as_str(),
            &execution.merge_report,
            execution.base_inputs.as_slice(),
            final_base_results.as_slice(),
        ) {
            Ok(batch) => {
                self.runtime.audio_capacity.record_success(
                    execution.base_inputs.len() + execution.recovery_inputs.len(),
                    duration_to_ms(workflow_started.elapsed()),
                );
                batch
            }
            Err(error) => {
                self.runtime.audio_capacity.record_failure();
                return Err(error);
            }
        };
        std::fs::create_dir_all(output.as_path()).map_err(|error| {
            format!(
                "failed to create audio document extract output dir {}: {error}",
                output.display()
            )
        })?;
        write_arrow_file(
            output.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME).as_path(),
            std::slice::from_ref(&batch),
        )?;
        write_audio_materialization_report(
            output.as_path(),
            &execution.base_materialized_shards,
            &execution.recovery_materialized_shards,
        )?;
        write_audio_transcript_admission_report(
            output.as_path(),
            &execution.transcript_admission_stats,
        )?;
        write_audio_cache_manifest(output.as_path(), &cache_manifest)?;
        mark_document_extract_cache_complete(output.as_path())?;
        Ok(DocumentExtractFlightRouteResponse::new(batch))
    }

    async fn connect_audio_shard_client(&self) -> Result<AudioShardFlightClient, String> {
        let endpoint_urls = self.audio_shard_endpoint_attempt_order()?;
        let mut last_error = None;
        for endpoint_url in endpoint_urls {
            match AudioShardFlightClient::connect(endpoint_url.as_str()).await {
                Ok(client) => return Ok(client),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            "audio shard endpoint pool did not produce a request attempt".to_owned()
        }))
    }

    fn audio_shard_endpoint_attempt_order(&self) -> Result<Vec<String>, String> {
        let default_endpoint = document_extract_default_endpoint_with_lookup(
            self.configured_default_endpoint.as_deref(),
            &|key| std::env::var(key).ok(),
        );
        let endpoint_urls = document_extract_endpoint_urls(default_endpoint.as_str());
        let request_index = self
            .runtime
            .endpoint_round_robin
            .fetch_add(1, Ordering::Relaxed);
        document_extract_endpoint_attempt_order_for_request(request_index, endpoint_urls.as_slice())
    }
}

fn audio_shard_request_options_for_document_extract(
    request: &DocumentExtractFlightRequest,
    config: &AudioDocumentExtractConfig,
) -> AudioShardFlightRequestOptions {
    AudioShardFlightRequestOptions {
        audio_worker: request.audio_worker.clone(),
        hosted_provider: request.audio_hosted_provider.clone(),
        hosted_base_url: request.audio_hosted_base_url.clone(),
        hosted_endpoint: request.audio_hosted_endpoint.clone(),
        hosted_model: request.audio_hosted_model.clone(),
        transcript_admission_dir: config.transcript_admission_dir.clone(),
        ..AudioShardFlightRequestOptions::default()
    }
}

fn duration_to_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn audio_cache_manifest(
    request: &DocumentExtractFlightRequest,
    config: &AudioDocumentExtractConfig,
    source_hash: &str,
    duration_ms: u64,
) -> serde_json::Value {
    let speech_segments_jsonl = config
        .speech_segments_jsonl_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    serde_json::json!({
        "schema": AUDIO_CACHE_MANIFEST_SCHEMA,
        "sourceSha256": source_hash,
        "sourceDurationMs": duration_ms,
        "backendProfile": config.backend_profile.as_str(),
        "chunkDurationMs": config.chunk_duration_ms,
        "contextBeforeMs": config.context_before_ms,
        "contextAfterMs": config.context_after_ms,
        "recoverySplitDurationMs": config.recovery_split_duration_ms,
        "sampleRateHz": config.sample_rate_hz,
        "channels": config.channels,
        "audioFormat": config.audio_format.as_str(),
        "speechMergeGapMs": config.speech_merge_gap_ms,
        "speechMinWindowMs": config.speech_min_window_ms,
        "speechLimitChunks": config.speech_limit_chunks,
        "speechSegmentsJsonl": speech_segments_jsonl,
        "requestAudioWorker": request.audio_worker.as_deref(),
        "hostedProvider": request.audio_hosted_provider.as_deref(),
        "hostedBaseUrl": request.audio_hosted_base_url.as_deref(),
        "hostedEndpoint": request.audio_hosted_endpoint.as_deref(),
        "hostedModel": request.audio_hosted_model.as_deref(),
    })
}

fn audio_cache_manifest_matches(output_dir: &Path, expected: &serde_json::Value) -> bool {
    let manifest_path = output_dir.join(AUDIO_CACHE_MANIFEST_NAME);
    let Ok(payload) = std::fs::read(manifest_path.as_path()) else {
        return false;
    };
    let Ok(actual) = serde_json::from_slice::<serde_json::Value>(payload.as_slice()) else {
        return false;
    };
    actual == *expected
}

fn read_cached_audio_document_batches(
    output_dir: &Path,
) -> Result<Option<Vec<arrow::record_batch::RecordBatch>>, String> {
    let marker_path = output_dir.join("_complete.marker");
    let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    if !marker_path.exists() || !resources_path.exists() {
        return Ok(None);
    }
    read_arrow_file(resources_path.as_path()).map(Some)
}

fn write_audio_cache_manifest(
    output_dir: &Path,
    manifest: &serde_json::Value,
) -> Result<(), String> {
    let manifest_path = output_dir.join(AUDIO_CACHE_MANIFEST_NAME);
    let payload = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("serialize audio document extract cache manifest: {error}"))?;
    std::fs::write(manifest_path.as_path(), payload).map_err(|error| {
        format!(
            "write audio document extract cache manifest `{}`: {error}",
            manifest_path.display()
        )
    })
}

fn write_audio_materialization_report(
    output_dir: &Path,
    base_shards: &[AudioShardMaterializedItem],
    recovery_shards: &[AudioShardMaterializedItem],
) -> Result<(), String> {
    let report_path = output_dir.join(AUDIO_MATERIALIZATION_REPORT_NAME);
    let source_counts =
        audio_materialization_source_counts(base_shards.iter().chain(recovery_shards.iter()));
    let artifact_cache_hit_count = source_counts
        .get("artifact-cache")
        .copied()
        .unwrap_or_default();
    let existing_output_count = source_counts
        .get("existing-output")
        .copied()
        .unwrap_or_default();
    let media_splitter_count = source_counts
        .get("media-splitter")
        .copied()
        .unwrap_or_default();
    let payload = serde_json::json!({
        "schema": AUDIO_MATERIALIZATION_REPORT_SCHEMA,
        "baseShardCount": base_shards.len(),
        "recoveryShardCount": recovery_shards.len(),
        "shardCount": base_shards.len() + recovery_shards.len(),
        "sourceCounts": source_counts,
        "artifactCacheHitCount": artifact_cache_hit_count,
        "existingOutputCount": existing_output_count,
        "mediaSplitterCount": media_splitter_count,
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("serialize audio materialization report: {error}"))?;
    std::fs::write(report_path.as_path(), bytes).map_err(|error| {
        format!(
            "write audio materialization report `{}`: {error}",
            report_path.display()
        )
    })
}

fn write_audio_transcript_admission_report(
    output_dir: &Path,
    stats: &AudioTranscriptAdmissionStats,
) -> Result<(), String> {
    let report_path = output_dir.join(AUDIO_TRANSCRIPT_ADMISSION_REPORT_NAME);
    let payload = serde_json::json!({
        "schema": AUDIO_TRANSCRIPT_ADMISSION_REPORT_SCHEMA,
        "enabled": stats.enabled,
        "hitCount": stats.hit_count,
        "missCount": stats.miss_count,
        "storedCount": stats.stored_count,
        "staleCount": stats.stale_count,
        "plannedHitCount": stats.planned_hit_count,
        "plannedMissCount": stats.planned_miss_count,
        "plannedStoredCount": stats.planned_stored_count,
        "plannedStaleCount": stats.planned_stale_count,
    });
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("serialize audio transcript admission report: {error}"))?;
    std::fs::write(report_path.as_path(), bytes).map_err(|error| {
        format!(
            "write audio transcript admission report `{}`: {error}",
            report_path.display()
        )
    })
}

fn audio_materialization_source_counts<'a>(
    shards: impl Iterator<Item = &'a AudioShardMaterializedItem>,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for shard in shards {
        *counts
            .entry(audio_materialization_source_key(
                shard.materialization_source,
            ))
            .or_insert(0) += 1;
    }
    counts
}

fn audio_materialization_source_key(source: AudioShardMaterializationSource) -> &'static str {
    match source {
        AudioShardMaterializationSource::ExistingOutput => "existing-output",
        AudioShardMaterializationSource::ArtifactCache => "artifact-cache",
        AudioShardMaterializationSource::MediaSplitter => "media-splitter",
    }
}

fn document_extract_audio_output_dir(source: &Path, output_dir: &str) -> PathBuf {
    if output_dir.trim().is_empty() {
        default_output_dir(source)
    } else {
        PathBuf::from(output_dir)
    }
}
