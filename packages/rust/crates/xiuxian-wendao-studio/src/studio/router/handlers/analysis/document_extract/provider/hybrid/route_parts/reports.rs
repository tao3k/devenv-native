use super::{
    BTreeMap, DocumentExtractFlightRequest, HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME,
    HYBRID_PAGE_OCR_TIMING_REPORT_NAME, HybridDocumentResourceBatch,
    Ocr2RegionMaterializationStats, PageRangeDoclingFallbackChunkTiming, Path,
    PdfOcrShardSchedulerTrace, failed_page_recovery_mode_label, is_hosted_vlm_direct_profile,
    ocr2_region_pipeline_mode_label, ocr2_region_render_chunk_mode_label,
};
use serde::Serialize;
use serde_json::{Map, Value, json};

pub(super) async fn write_hybrid_page_ocr_fallback_report(
    request: &DocumentExtractFlightRequest,
    output: &Path,
    reason: &str,
) {
    let report = json!({
        "schema": "xiuxian_wendao.hybrid_page_ocr_fallback.v1",
        "sourcePath": request.source_path.as_str(),
        "outputDir": output.to_string_lossy(),
        "reason": reason,
        "fullDoclingFallbackCount": 1,
    });
    let path = output.join(HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME);
    match serde_json::to_vec_pretty(&report) {
        Ok(bytes) => {
            if let Err(error) = tokio::fs::write(path.as_path(), bytes).await {
                log::warn!(
                    "failed to write hybrid PDF OCR fallback report `{}`: {error}",
                    path.display()
                );
            }
        }
        Err(error) => {
            log::warn!("failed to serialize hybrid PDF OCR fallback report: {error}");
        }
    }
}

pub(super) fn page_range_docling_fallback_chunk_summary(
    chunks: &[PageRangeDoclingFallbackChunkTiming],
) -> serde_json::Value {
    let longest = chunks
        .iter()
        .max_by(|left, right| left.elapsed_ms.total_cmp(&right.elapsed_ms));
    let elapsed_ms_total = chunks.iter().map(|chunk| chunk.elapsed_ms).sum::<f64>();
    let document_timing_total_elapsed_ms = chunks
        .iter()
        .filter_map(|chunk| chunk.document_timing_total_elapsed_ms)
        .sum::<f64>();
    let mut document_timing_phase_elapsed_ms = BTreeMap::new();
    let mut document_extract_profile_counts = BTreeMap::new();
    for chunk in chunks {
        *document_extract_profile_counts
            .entry(chunk.document_extract_profile.clone())
            .or_insert(0usize) += 1;
        for (phase, elapsed_ms) in &chunk.document_timing_phase_elapsed_ms {
            *document_timing_phase_elapsed_ms
                .entry(phase.clone())
                .or_insert(0.0) += elapsed_ms;
        }
    }
    let elapsed_ms_min = chunks
        .iter()
        .map(|chunk| chunk.elapsed_ms)
        .min_by(f64::total_cmp);
    let elapsed_ms_mean = (!chunks.is_empty()).then(|| {
        let chunk_count = u32::try_from(chunks.len()).map_or(f64::from(u32::MAX), f64::from);
        elapsed_ms_total / chunk_count
    });
    let elapsed_ms_max = longest.map(|chunk| chunk.elapsed_ms);
    json!({
        "chunkCount": chunks.len(),
        "elapsedMsMax": elapsed_ms_max,
        "elapsedMsMin": elapsed_ms_min,
        "elapsedMsMean": elapsed_ms_mean,
        "elapsedMsSpread": elapsed_ms_max.zip(elapsed_ms_min).map(|(max, min)| max - min),
        "elapsedMsMaxToMeanRatio": elapsed_ms_max
            .zip(elapsed_ms_mean)
            .filter(|(_, mean)| *mean > 0.0)
            .map(|(max, mean)| max / mean),
        "elapsedMsTotal": elapsed_ms_total,
        "documentTimingTotalElapsedMs": document_timing_total_elapsed_ms,
        "documentTimingPhaseElapsedMs": document_timing_phase_elapsed_ms,
        "documentExtractProfileCounts": document_extract_profile_counts,
        "resourceRows": chunks.iter().map(|chunk| chunk.resource_rows).sum::<usize>(),
        "sourceProfilePageCount": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.page_count)
            .sum::<usize>(),
        "sourceProfileEstimatedWeightTotal": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.estimated_weight_total)
            .sum::<u64>(),
        "sourceProfileEstimatedStructureCostTotal": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.estimated_structure_cost_total)
            .sum::<u64>(),
        "sourceProfileEstimatedStructureCostMax": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.estimated_structure_cost_max)
            .max()
            .unwrap_or(0),
        "sourceProfileStructureAuthorityRequiredCount": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.structure_authority_required_count)
            .sum::<usize>(),
        "sourceProfileFastProfileRiskCount": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.fast_profile_risk_count)
            .sum::<usize>(),
        "sourceProfileBackendTextTopupCount": chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.backend_text_topup_count)
            .sum::<usize>(),
        "hedgedChunkCount": chunks.iter().filter(|chunk| chunk.hedged).count(),
        "attemptCountTotal": chunks.iter().map(|chunk| chunk.attempt_count).sum::<usize>(),
        "longestPageStart": longest.map(|chunk| chunk.page_start),
        "longestPageEnd": longest.map(|chunk| chunk.page_end),
        "longestOneBasedStart": longest.map(|chunk| chunk.one_based_start),
        "longestOneBasedEnd": longest.map(|chunk| chunk.one_based_end),
        "longestResourceRows": longest.map(|chunk| chunk.resource_rows),
        "longestDocumentTimingTotalElapsedMs": longest
            .and_then(|chunk| chunk.document_timing_total_elapsed_ms),
        "longestDocumentTimingPhaseElapsedMs": longest
            .map(|chunk| &chunk.document_timing_phase_elapsed_ms),
        "longestSourceProfile": longest.and_then(|chunk| chunk.source_profile.as_ref()),
    })
}

pub(super) fn docling_centered_structure_authority_page_count(
    resource_batch: &HybridDocumentResourceBatch,
) -> usize {
    resource_batch
        .ocr_inputs
        .iter()
        .filter(|input| {
            input.shard_type == "page" && input.ocr_profile == "docling-compatible-page-ocr-v1"
        })
        .count()
        + resource_batch
            .page_range_docling_fallback_chunks
            .iter()
            .filter_map(|chunk| chunk.source_profile.as_ref())
            .map(|profile| profile.structure_authority_required_count)
            .sum::<usize>()
}

pub(super) fn docling_centered_text_shortcut_page_count(
    resource_batch: &HybridDocumentResourceBatch,
) -> usize {
    resource_batch
        .ocr_inputs
        .iter()
        .filter(|input| {
            input.shard_type == "page"
                && matches!(
                    input.ocr_profile.as_str(),
                    "docling-backend-text-ocr-v1" | "docling-fast-text-ocr"
                )
        })
        .count()
}

pub(super) fn docling_centered_ocr_patch_region_count(
    resource_batch: &HybridDocumentResourceBatch,
) -> usize {
    resource_batch
        .ocr_inputs
        .iter()
        .filter(|input| {
            input.shard_type == "region" && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        })
        .count()
}

pub(super) async fn write_hybrid_page_ocr_timing_report(
    source: &Path,
    output: &Path,
    resource_batch: &HybridDocumentResourceBatch,
    region_materialization_stats: &Ocr2RegionMaterializationStats,
    phase_elapsed_ms: &BTreeMap<String, f64>,
    scheduler_trace: &[PdfOcrShardSchedulerTrace],
    total_elapsed_ms: f64,
) {
    let mut report = Map::new();
    insert_report_value(
        &mut report,
        "schema",
        "xiuxian_wendao.hybrid_page_ocr_timing.v1",
    );
    insert_report_value(
        &mut report,
        "sourcePath",
        source.to_string_lossy().to_string(),
    );
    insert_report_value(
        &mut report,
        "outputDir",
        output.to_string_lossy().to_string(),
    );
    insert_report_value(&mut report, "pageCount", resource_batch.page_count);
    insert_report_value(
        &mut report,
        "ocrShardCount",
        resource_batch.ocr_inputs.len(),
    );
    insert_report_value(
        &mut report,
        "ocr2RegionShardCount",
        resource_batch
            .ocr_inputs
            .iter()
            .filter(|input| {
                input.shard_type == "region"
                    && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
            })
            .count(),
    );
    insert_region_materialization_stats(&mut report, region_materialization_stats);
    insert_report_value(
        &mut report,
        "ocr2RegionPipelineMode",
        ocr2_region_pipeline_mode_label(),
    );
    insert_report_value(
        &mut report,
        "ocr2RegionRenderChunkMode",
        ocr2_region_render_chunk_mode_label(),
    );
    insert_report_value(
        &mut report,
        "failedPageRecoveryMode",
        failed_page_recovery_mode_label(),
    );
    insert_report_value(
        &mut report,
        "structureAuthorityPages",
        docling_centered_structure_authority_page_count(resource_batch),
    );
    insert_report_value(
        &mut report,
        "textShortcutPages",
        docling_centered_text_shortcut_page_count(resource_batch),
    );
    insert_report_value(
        &mut report,
        "ocrPatchRegions",
        docling_centered_ocr_patch_region_count(resource_batch),
    );
    insert_report_value(
        &mut report,
        "pageRangeDoclingFallbackPages",
        resource_batch.page_range_docling_fallback_pages.len(),
    );
    insert_report_value(
        &mut report,
        "pageRangeDoclingFallbackChunkCount",
        resource_batch.page_range_docling_fallback_chunks.len(),
    );
    insert_report_value(
        &mut report,
        "pageRangeDoclingFallbackPlan",
        &resource_batch.page_range_docling_fallback_plan,
    );
    insert_report_value(
        &mut report,
        "pageRangeDoclingFallbackChunks",
        &resource_batch.page_range_docling_fallback_chunks,
    );
    report.insert(
        "pageRangeDoclingFallbackChunkSummary".to_string(),
        page_range_docling_fallback_chunk_summary(
            resource_batch.page_range_docling_fallback_chunks.as_slice(),
        ),
    );
    insert_report_value(&mut report, "fullDoclingFallbackCount", 0);
    insert_report_value(
        &mut report,
        "failedPageRecoveryHostedVlmPageShardCount",
        resource_batch
            .ocr_inputs
            .iter()
            .filter(|input| {
                input.shard_type == "page"
                    && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
            })
            .count(),
    );
    insert_report_value(&mut report, "ocrSchedulerTrace", scheduler_trace);
    insert_report_value(&mut report, "totalElapsedMs", total_elapsed_ms);
    insert_report_value(&mut report, "phaseElapsedMs", phase_elapsed_ms);
    let report = Value::Object(report);
    let path = output.join(HYBRID_PAGE_OCR_TIMING_REPORT_NAME);
    match serde_json::to_vec_pretty(&report) {
        Ok(bytes) => {
            if let Err(error) = tokio::fs::write(path.as_path(), bytes).await {
                log::warn!(
                    "failed to write hybrid PDF OCR timing report `{}`: {error}",
                    path.display()
                );
            }
        }
        Err(error) => {
            log::warn!("failed to serialize hybrid PDF OCR timing report: {error}");
        }
    }
}

fn insert_region_materialization_stats(
    report: &mut Map<String, Value>,
    stats: &Ocr2RegionMaterializationStats,
) {
    insert_report_value(
        report,
        "ocr2RegionRequestCount",
        stats.requested_region_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderedShardCount",
        stats.rendered_region_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderCacheHitCount",
        stats.render_cache_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderCacheMissCount",
        stats.render_cache_miss_count,
    );
    insert_artifact_cache_stats(report, stats);
    insert_report_value(
        report,
        "ocr2RegionRenderReportedElapsedMs",
        stats.render_reported_elapsed_ms,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelinePlannedRenderChunkCount",
        stats.pipeline_planned_render_chunk_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineEndpointCount",
        stats.pipeline_endpoint_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRenderAheadLimit",
        stats.pipeline_render_ahead_limit,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRenderSpawnCount",
        stats.pipeline_render_spawn_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRenderChunkCount",
        stats.pipeline_render_chunk_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRegionDispatchCount",
        stats.pipeline_region_dispatch_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineBaseResultCount",
        stats.pipeline_base_result_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineBaseResultShardCount",
        stats.pipeline_base_result_shard_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRegionResultCount",
        stats.pipeline_region_result_count,
    );
    insert_report_value(
        report,
        "ocr2RegionPipelineRegionResultShardCount",
        stats.pipeline_region_result_shard_count,
    );
}

fn insert_artifact_cache_stats(
    report: &mut Map<String, Value>,
    stats: &Ocr2RegionMaterializationStats,
) {
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheHitCount",
        stats.render_artifact_cache_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheMissCount",
        stats.render_artifact_cache_miss_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheThrottledCount",
        stats.render_artifact_cache_throttled_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheByteCount",
        stats.render_artifact_cache_byte_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCachePageRasterHitCount",
        stats.render_artifact_cache_page_raster_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCachePageRasterMissCount",
        stats.render_artifact_cache_page_raster_miss_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCachePageRasterThrottledCount",
        stats.render_artifact_cache_page_raster_throttled_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCachePageRasterByteCount",
        stats.render_artifact_cache_page_raster_byte_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionCropHitCount",
        stats.render_artifact_cache_region_crop_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionCropMissCount",
        stats.render_artifact_cache_region_crop_miss_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionCropThrottledCount",
        stats.render_artifact_cache_region_crop_throttled_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionCropByteCount",
        stats.render_artifact_cache_region_crop_byte_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionHitCount",
        stats.render_artifact_cache_region_manifest_projection_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionMissCount",
        stats.render_artifact_cache_region_manifest_projection_miss_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionThrottledCount",
        stats.render_artifact_cache_region_manifest_projection_throttled_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionByteCount",
        stats.render_artifact_cache_region_manifest_projection_byte_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionRowHitCount",
        stats.render_artifact_cache_region_manifest_projection_row_hit_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionRowMissCount",
        stats.render_artifact_cache_region_manifest_projection_row_miss_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionRowThrottledCount",
        stats.render_artifact_cache_region_manifest_projection_row_throttled_count,
    );
    insert_report_value(
        report,
        "ocr2RegionRenderArtifactCacheRegionManifestProjectionRowByteCount",
        stats.render_artifact_cache_region_manifest_projection_row_byte_count,
    );
}

fn insert_report_value<T>(report: &mut Map<String, Value>, key: &'static str, value: T)
where
    T: Serialize,
{
    match serde_json::to_value(value) {
        Ok(value) => {
            report.insert(key.to_string(), value);
        }
        Err(error) => {
            log::warn!("failed to serialize hybrid PDF OCR timing report field `{key}`: {error}");
            report.insert(key.to_string(), Value::Null);
        }
    }
}
