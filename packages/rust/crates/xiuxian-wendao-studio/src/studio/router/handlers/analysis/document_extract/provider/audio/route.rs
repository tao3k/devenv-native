//! Studio document-extract route for Rust-planned audio shards.

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use xiuxian_wendao_attachments::audio::{
    AudioRecoveryPatchGateOptions, AudioRiskParentSelectionOptions, AudioShardWorkerProfile,
};
use xiuxian_wendao_server::transport::{
    DocumentExtractFlightRequest, DocumentExtractFlightRouteResponse,
};

use super::config::AudioDocumentExtractConfig;
use super::plan::{
    audio_materialization_input, build_full_coverage_audio_plan, probe_audio_duration_ms,
    source_sha256_hex,
};
use super::response::build_audio_transcript_batch;
use super::speech::recovery_speech_window_input_from_config;
use crate::studio::document_extract_audio_client::{
    AudioShardFlightClient, AudioShardRecoveryWorkflowRequest,
};
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, mark_document_extract_cache_complete,
    read_cached_document_batches, write_arrow_file,
};
use crate::studio::router::handlers::analysis::document_extract::provider::StudioDocumentExtractFlightRouteProvider;
use crate::studio::router::handlers::analysis::document_extract::provider::transport::{
    document_extract_default_endpoint_with_lookup,
    document_extract_endpoint_attempt_order_for_request, document_extract_endpoint_urls,
};
use crate::studio::router::handlers::analysis::document_extract::registry::default_output_dir;

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
        if source.exists()
            && !request.force
            && let Some(batches) = read_cached_document_batches(source.as_path(), output.as_path())?
        {
            return Ok(DocumentExtractFlightRouteResponse::from_batches(batches));
        }

        let duration_ms = probe_audio_duration_ms(source.as_path(), config.ffprobe_path.as_path())?;
        let source_hash = source_sha256_hex(source.as_path())?;
        let plan =
            build_full_coverage_audio_plan(source.as_path(), source_hash, duration_ms, &config)?;
        let materialization =
            audio_materialization_input(source.clone(), output.as_path(), &config, request.force);
        let recovery_speech_window_input =
            recovery_speech_window_input_from_config(&plan, &config)?;
        let profile = AudioShardWorkerProfile::transcription(config.backend_profile.as_str());
        let client = self.connect_audio_shard_client().await?;
        let _permit = self.acquire_document_extract_dispatch_permit().await?;
        let execution = client
            .execute_recovery_split(AudioShardRecoveryWorkflowRequest {
                parent_plan: &plan,
                materialization: &materialization,
                profile: &profile,
                request_metrics: &[],
                selection_options: AudioRiskParentSelectionOptions::default(),
                patch_options: AudioRecoveryPatchGateOptions::default(),
                recovery_split_duration_ms: config.recovery_split_duration_ms,
                recovery_speech_window_input: recovery_speech_window_input.as_ref(),
                base_worker_budget: config.base_worker_budget,
                recovery_worker_budget: config.recovery_worker_budget,
            })
            .await?;
        let output_string = output.to_string_lossy().to_string();
        let batch = build_audio_transcript_batch(
            request.source_path.as_str(),
            output_string.as_str(),
            &execution.merge_report,
        )?;
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

fn document_extract_audio_output_dir(source: &Path, output_dir: &str) -> PathBuf {
    if output_dir.trim().is_empty() {
        default_output_dir(source)
    } else {
        PathBuf::from(output_dir)
    }
}
