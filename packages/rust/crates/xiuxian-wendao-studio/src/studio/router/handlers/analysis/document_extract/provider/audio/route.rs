//! Studio document-extract route for Rust-planned audio shards.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use xiuxian_db_store::artifact_cache::ArtifactBlobCacheBackendConfig;
use xiuxian_llm::model_routing::{
    WendaoAttachmentRouteConfig, WendaoAttachmentRouteInput, WendaoModelDecision,
    WendaoModelRoutingMode, WendaoRouteIntent, wendao_attachment_model_route_decision,
};
use xiuxian_qianji::WorkflowTrace;
use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions, AudioShardMaterializationInput,
    AudioShardMaterializationSource, AudioShardMaterializedItem, AudioShardPlan,
    AudioShardWorkerProfile, AudioSpeechWindowPlannerInput, AudioTranscriptAdmissionStats,
    apply_audio_recovery_patch_decisions,
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
use super::speech::{
    base_speech_window_plan_from_config, configured_speech_segments_sha256_from_config,
    recovery_speech_window_input_from_config,
};
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardFlightRequestOptions, AudioShardRecoveryWorkflowExecution,
    AudioShardRecoveryWorkflowRequest,
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
const AUDIO_ROUTE_TASK_KIND: &str = "attachment-extract";
const AUDIO_ROUTE_MODALITY: &str = "audio";
const AUDIO_ROUTE_SOURCE_KIND: &str = "attachment";
const AUDIO_ROUTE_PRECISION_TIER: &str = "high";
const AUDIO_ROUTE_PRIVACY_TIER: &str = "private";
const AUDIO_ROUTE_EVIDENCE_PROFILE: &str = "audio-transcript";
const AUDIO_ROUTE_MIN_LATENCY_BUDGET_MS: u64 = 60_000;

struct AudioRecoveryWorkflowDispatch<'a> {
    request: &'a DocumentExtractFlightRequest,
    config: &'a AudioDocumentExtractConfig,
    plan: &'a AudioShardPlan,
    materialization: &'a AudioShardMaterializationInput,
    recovery_speech_window_input: Option<&'a AudioSpeechWindowPlannerInput>,
    base_worker_budget: Option<usize>,
    recovery_worker_budget: Option<usize>,
    route_intent: Option<&'a WendaoRouteIntent>,
    model_decision: Option<&'a WendaoModelDecision>,
}

impl StudioDocumentExtractFlightRouteProvider {
    pub(crate) async fn audio_shards_document_extract_batch(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let model_routing_config = self.model_routing_config()?;
        self.audio_shards_document_extract_batch_with_config(
            request,
            AudioDocumentExtractConfig::from_model_routing_config(model_routing_config.as_ref())?,
        )
        .await
    }

    pub(crate) async fn audio_shards_document_extract_batch_for_source_hash(
        &self,
        request: &DocumentExtractFlightRequest,
        source_hash: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let model_routing_config = self.model_routing_config()?;
        self.audio_shards_document_extract_batch_with_config_and_source_hash(
            request,
            AudioDocumentExtractConfig::from_model_routing_config(model_routing_config.as_ref())?,
            source_hash,
        )
        .await
    }

    pub(crate) async fn audio_shards_document_extract_batch_with_config(
        &self,
        request: &DocumentExtractFlightRequest,
        config: AudioDocumentExtractConfig,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(request.source_path.as_str());
        let source_hash = source_sha256_hex(source.as_path())?;
        self.audio_shards_document_extract_batch_with_config_and_source_hash(
            request,
            config,
            source_hash.as_str(),
        )
        .await
    }

    pub(crate) async fn audio_shards_document_extract_batch_with_config_and_source_hash(
        &self,
        request: &DocumentExtractFlightRequest,
        config: AudioDocumentExtractConfig,
        source_hash: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        let source = PathBuf::from(request.source_path.as_str());
        let output = document_extract_audio_output_dir(&source, request.output_dir.as_str());
        let duration_ms = probe_audio_duration_ms(source.as_path(), config.ffprobe_path.as_path())?;
        let source_hash = normalized_source_hash(source_hash)?;
        let full_coverage_plan = build_full_coverage_audio_plan(
            source.as_path(),
            source_hash.clone(),
            duration_ms,
            &config,
        )?;
        let plan = base_speech_window_plan_from_config(&full_coverage_plan, &config)?
            .unwrap_or(full_coverage_plan);
        let model_route = audio_model_route_decision_for_document_extract(
            request,
            &config,
            &plan,
            source_hash.as_str(),
            duration_ms,
        )
        .await?;
        let cache_manifest = audio_cache_manifest(
            request,
            &config,
            source_hash.as_str(),
            duration_ms,
            model_route.as_ref(),
        )?;
        if source.exists()
            && !request.force
            && audio_cache_manifest_matches(output.as_path(), &cache_manifest)
            && let Some(batches) = read_cached_audio_document_batches(output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }
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
        let execution = self
            .execute_audio_recovery_workflow(
                &client,
                &profile,
                AudioRecoveryWorkflowDispatch {
                    request,
                    config: &config,
                    plan: &plan,
                    materialization: &materialization,
                    recovery_speech_window_input: recovery_speech_window_input.as_ref(),
                    base_worker_budget: scheduled_base_worker_budget,
                    recovery_worker_budget: scheduled_recovery_worker_budget,
                    route_intent: model_route.as_ref().map(|(intent, _)| intent),
                    model_decision: model_route.as_ref().map(|(_, decision)| decision),
                },
            )
            .await?;
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
            &config,
            &execution.base_materialized_shards,
            &execution.recovery_materialized_shards,
            &execution.trace,
        )?;
        write_audio_transcript_admission_report(
            output.as_path(),
            &execution.transcript_admission_stats,
        )?;
        write_audio_cache_manifest(output.as_path(), &cache_manifest)?;
        mark_document_extract_cache_complete(output.as_path())?;
        Ok(DocumentExtractFlightRouteResponse::new(batch))
    }

    async fn execute_audio_recovery_workflow(
        &self,
        client: &AudioShardFlightClient,
        profile: &AudioShardWorkerProfile,
        dispatch: AudioRecoveryWorkflowDispatch<'_>,
    ) -> Result<AudioShardRecoveryWorkflowExecution, String> {
        match client
            .execute_recovery_split_with_options(
                AudioShardRecoveryWorkflowRequest {
                    parent_plan: dispatch.plan,
                    materialization: dispatch.materialization,
                    profile,
                    request_metrics: &[],
                    selection_options: audio_recovery_selection_options_for_plan(dispatch.plan),
                    patch_options: AudioRecoveryPatchGateOptions::default(),
                    recovery_split_duration_ms: dispatch.config.recovery_split_duration_ms,
                    recovery_speech_window_input: dispatch.recovery_speech_window_input,
                    base_worker_budget: dispatch.base_worker_budget,
                    recovery_worker_budget: dispatch.recovery_worker_budget,
                },
                audio_shard_request_options_for_document_extract(
                    dispatch.request,
                    dispatch.config,
                    dispatch.route_intent,
                    dispatch.model_decision,
                ),
            )
            .await
        {
            Ok(execution) => Ok(execution),
            Err(error) => {
                self.runtime.audio_capacity.record_failure();
                Err(error)
            }
        }
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

pub(crate) fn audio_recovery_selection_options_for_plan(
    plan: &AudioShardPlan,
) -> AudioRiskParentSelectionOptions {
    if plan.strategy != "speech-segments" {
        return AudioRiskParentSelectionOptions::default();
    }
    AudioRiskParentSelectionOptions {
        include_boundaries: false,
        max_chars_per_minute: -1.0,
        max_chinese_ratio: -1.0,
        min_latency_ms: u64::MAX,
        min_repeated_ngram_ratio: 2.0,
        ..AudioRiskParentSelectionOptions::default()
    }
}

fn audio_shard_request_options_for_document_extract(
    request: &DocumentExtractFlightRequest,
    config: &AudioDocumentExtractConfig,
    route_intent: Option<&WendaoRouteIntent>,
    model_decision: Option<&WendaoModelDecision>,
) -> AudioShardFlightRequestOptions {
    AudioShardFlightRequestOptions {
        audio_worker: request.audio_worker.clone(),
        hosted_provider: request.audio_hosted_provider.clone(),
        hosted_base_url: request.audio_hosted_base_url.clone(),
        hosted_endpoint: request.audio_hosted_endpoint.clone(),
        hosted_model: request.audio_hosted_model.clone(),
        route_intent: route_intent.cloned(),
        model_decision: model_decision.cloned(),
        transcript_admission_dir: config.transcript_admission_dir.clone(),
        ..AudioShardFlightRequestOptions::default()
    }
}

async fn audio_model_route_decision_for_document_extract(
    request: &DocumentExtractFlightRequest,
    config: &AudioDocumentExtractConfig,
    plan: &AudioShardPlan,
    source_hash: &str,
    duration_ms: u64,
) -> Result<Option<(WendaoRouteIntent, WendaoModelDecision)>, String> {
    let route_provider = request
        .audio_hosted_provider
        .as_ref()
        .or(config.route_provider.as_ref())
        .cloned();
    let route_config = WendaoAttachmentRouteConfig {
        route_provider,
        route_model: config.route_model.clone(),
        backend_profile: config.backend_profile.clone(),
        model_routing_mode: config.model_routing_mode,
        vllm_sr_base_url: config.vllm_sr_base_url.clone(),
    };
    let route_input =
        audio_route_input_for_document_extract(config, plan, source_hash, duration_ms);
    wendao_attachment_model_route_decision(&route_config, &route_input)
        .await
        .map(Some)
}

fn audio_route_input_for_document_extract(
    config: &AudioDocumentExtractConfig,
    plan: &AudioShardPlan,
    source_hash: &str,
    duration_ms: u64,
) -> WendaoAttachmentRouteInput {
    WendaoAttachmentRouteInput {
        task_kind: AUDIO_ROUTE_TASK_KIND.to_owned(),
        modality: AUDIO_ROUTE_MODALITY.to_owned(),
        source_kind: AUDIO_ROUTE_SOURCE_KIND.to_owned(),
        precision_tier: AUDIO_ROUTE_PRECISION_TIER.to_owned(),
        privacy_tier: AUDIO_ROUTE_PRIVACY_TIER.to_owned(),
        latency_budget_ms: audio_route_latency_budget_ms(duration_ms),
        evidence_profile: AUDIO_ROUTE_EVIDENCE_PROFILE.to_owned(),
        artifact_refs: vec![
            format!("source-sha256:{source_hash}"),
            format!("duration-ms:{duration_ms}"),
            format!("shard-count:{}", plan.start_offsets_ms.len()),
            format!("plan-strategy:{}", plan.strategy.as_str()),
            format!("backend-profile:{}", config.backend_profile.as_str()),
        ],
    }
}

fn audio_route_latency_budget_ms(duration_ms: u64) -> u64 {
    AUDIO_ROUTE_MIN_LATENCY_BUDGET_MS.max(duration_ms.saturating_mul(2))
}

fn normalized_source_hash(source_hash: &str) -> Result<String, String> {
    let normalized = source_hash.trim();
    if normalized.is_empty() {
        Err("audio source sha256 must be non-empty".to_owned())
    } else {
        Ok(normalized.to_owned())
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
    model_route: Option<&(WendaoRouteIntent, WendaoModelDecision)>,
) -> Result<serde_json::Value, String> {
    let speech_segments_jsonl = config
        .speech_segments_jsonl_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());
    let speech_segments_sha256 = configured_speech_segments_sha256_from_config(config)?;
    let (route_selected_provider, route_selected_model, route_selected_backend_profile) =
        match model_route {
            Some((_, decision)) => (
                Some(decision.selected_provider.as_str()),
                Some(decision.selected_model.as_str()),
                Some(decision.selected_backend_profile.as_str()),
            ),
            None => (None, None, None),
        };
    Ok(serde_json::json!({
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
        "audioBitrate": config.audio_bitrate.as_deref(),
        "speechMergeGapMs": config.speech_merge_gap_ms,
        "speechMinWindowMs": config.speech_min_window_ms,
        "speechLimitChunks": config.speech_limit_chunks,
        "speechSegmentsJsonl": speech_segments_jsonl,
        "speechSegmentsSha256": speech_segments_sha256,
        "modelRoutingMode": audio_model_routing_mode_name(config.model_routing_mode),
        "routeProviderHint": config.route_provider.as_deref(),
        "routeSelectedProvider": route_selected_provider,
        "routeSelectedModel": route_selected_model,
        "routeSelectedBackendProfile": route_selected_backend_profile,
        "requestAudioWorker": request.audio_worker.as_deref(),
        "hostedProvider": request.audio_hosted_provider.as_deref(),
        "hostedBaseUrl": request.audio_hosted_base_url.as_deref(),
        "hostedEndpoint": request.audio_hosted_endpoint.as_deref(),
        "hostedModel": request.audio_hosted_model.as_deref(),
    }))
}

fn audio_model_routing_mode_name(mode: WendaoModelRoutingMode) -> &'static str {
    match mode {
        WendaoModelRoutingMode::VllmSr => "vllm-sr",
        WendaoModelRoutingMode::Deterministic => "deterministic",
    }
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
    config: &AudioDocumentExtractConfig,
    base_shards: &[AudioShardMaterializedItem],
    recovery_shards: &[AudioShardMaterializedItem],
    workflow_trace: &WorkflowTrace,
) -> Result<(), String> {
    let report_path = output_dir.join(AUDIO_MATERIALIZATION_REPORT_NAME);
    let shards = base_shards
        .iter()
        .chain(recovery_shards.iter())
        .collect::<Vec<_>>();
    let source_counts = audio_materialization_source_counts(shards.iter().copied());
    let source_bytes = audio_materialization_source_bytes(shards.iter().copied());
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
    let artifact_cache_hit_bytes = source_bytes
        .get("artifact-cache")
        .copied()
        .unwrap_or_default();
    let existing_output_bytes = source_bytes
        .get("existing-output")
        .copied()
        .unwrap_or_default();
    let media_splitter_bytes = source_bytes
        .get("media-splitter")
        .copied()
        .unwrap_or_default();
    let payload = serde_json::json!({
        "schema": AUDIO_MATERIALIZATION_REPORT_SCHEMA,
        "artifactCache": audio_artifact_cache_report(config),
        "baseShardCount": base_shards.len(),
        "recoveryShardCount": recovery_shards.len(),
        "shardCount": base_shards.len() + recovery_shards.len(),
        "sourceCounts": source_counts,
        "sourceBytes": source_bytes,
        "byteCount": artifact_cache_hit_bytes
            .saturating_add(existing_output_bytes)
            .saturating_add(media_splitter_bytes),
        "artifactCacheHitCount": artifact_cache_hit_count,
        "artifactCacheHitBytes": artifact_cache_hit_bytes,
        "existingOutputCount": existing_output_count,
        "existingOutputBytes": existing_output_bytes,
        "mediaSplitterCount": media_splitter_count,
        "mediaSplitterBytes": media_splitter_bytes,
        "workflow": audio_materialization_workflow_report(workflow_trace),
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

fn audio_materialization_workflow_report(workflow_trace: &WorkflowTrace) -> serde_json::Value {
    let stage_elapsed_ms = workflow_trace
        .stages
        .iter()
        .map(|stage| {
            (
                stage.stage_id.as_str(),
                nanos_to_millis(stage.duration_nanos),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let total_elapsed_ms = workflow_trace
        .stages
        .iter()
        .map(|stage| stage.duration_nanos)
        .fold(0_u64, u64::saturating_add);
    serde_json::json!({
        "workflowId": workflow_trace.workflow_id.as_str(),
        "stageCount": workflow_trace.stages.len(),
        "stageElapsedMs": stage_elapsed_ms,
        "totalElapsedMs": nanos_to_millis(total_elapsed_ms),
    })
}

fn nanos_to_millis(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn audio_artifact_cache_report(config: &AudioDocumentExtractConfig) -> serde_json::Value {
    let Some(root) = config.artifact_cache_dir.as_ref() else {
        return serde_json::json!({
            "configured": false,
        });
    };
    match ArtifactBlobCacheBackendConfig::from_root_and_env(root) {
        Ok(cache_config) => serde_json::json!({
            "configured": true,
            "backend": cache_config.kind().as_str(),
            "root": cache_config.root().to_string_lossy(),
            "memoryBytes": cache_config.memory_capacity_bytes(),
            "storageBytes": cache_config.storage_capacity_bytes(),
            "runtimeWorkers": cache_config.runtime_worker_threads(),
            "memoryShards": cache_config.memory_shards(),
            "recoverConcurrency": cache_config.recover_concurrency(),
            "flushers": cache_config.flushers(),
            "reclaimers": cache_config.reclaimers(),
            "memoryWeighter": cache_config.foyer_memory_weighter_name(),
            "policy": cache_config.foyer_cache_policy_name(),
            "blockSizeBytes": cache_config.foyer_block_size_bytes(),
        }),
        Err(error) => serde_json::json!({
            "configured": true,
            "root": root.to_string_lossy(),
            "configError": error.to_string(),
        }),
    }
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

fn audio_materialization_source_bytes<'a>(
    shards: impl Iterator<Item = &'a AudioShardMaterializedItem>,
) -> BTreeMap<&'static str, u64> {
    let mut bytes = BTreeMap::new();
    for shard in shards {
        *bytes
            .entry(audio_materialization_source_key(
                shard.materialization_source,
            ))
            .or_insert(0) += shard.shard_byte_len;
    }
    bytes
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

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/audio/route_model_routing.rs"]
mod model_routing_tests;
#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/audio/route_materialization_report.rs"]
mod tests;
