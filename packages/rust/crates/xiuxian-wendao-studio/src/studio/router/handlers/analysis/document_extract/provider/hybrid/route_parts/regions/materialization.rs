#[cfg(feature = "document-extract-pdf-render")]
use std::collections::BTreeMap;
#[cfg(feature = "document-extract-pdf-render")]
use std::path::PathBuf;
#[cfg(feature = "document-extract-pdf-render")]
use std::time::Instant;
use std::{collections::BTreeSet, path::Path};

#[cfg(feature = "document-extract-pdf-render")]
use serde_json::{Value, json};
#[cfg(feature = "document-extract-pdf-render")]
use sha2::{Digest, Sha256};

use xiuxian_wendao_attachments::pdf::ocr::{PdfOcrShardInput, is_hosted_vlm_direct_profile};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::ocr::{
    decode_ocr_shard_input_batches, merge_hosted_vlm_recovery_region_inputs,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::profile::{
    PdfSourcePageProfile, source_pdf_page_profiles_cached,
};
use xiuxian_wendao_attachments::pdf::render::PdfPageRenderShardReport;
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfRegionShardRenderRequest,
    PdfRenderRoutingDecision, PdfRenderStatus,
};
#[cfg(feature = "document-extract-pdf-render")]
use xiuxian_wendao_attachments::pdf::render::{
    render_pdf_page_shards_for_page_indices, render_pdf_region_shards_with_source_hash,
};

#[cfg(feature = "document-extract-pdf-render")]
use crate::studio::router::handlers::analysis::document_extract::arrow_cache::read_arrow_file;
#[cfg(feature = "document-extract-pdf-render")]
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::hybrid_page_ocr_render_profile_with_lookup;
#[cfg(feature = "document-extract-pdf-render")]
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::render::{
    automatic_ocr2_recovery_region_requests_for_source_with_lookup,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_region_requests_for_source_with_lookup,
};
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::route::support::Ocr2RegionMaterialization;
#[cfg(feature = "document-extract-pdf-render")]
use crate::studio::router::handlers::analysis::document_extract::provider::hybrid::types::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};

#[cfg(feature = "document-extract-pdf-render")]
const DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV: &str =
    "WENDAO_DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT";
#[cfg(feature = "document-extract-pdf-render")]
const OCR2_REGION_RENDER_CACHE_DIR_NAME: &str = "hosted-vlm-region-renders";
#[cfg(feature = "document-extract-pdf-render")]
const OCR_SHARD_MANIFEST_ARROW_NAME: &str = "_ocr_shards.arrow";
#[cfg(feature = "document-extract-pdf-render")]
const OCR_SHARD_INPUT_ARROW_NAME: &str = "_ocr_input.arrow";
#[cfg(feature = "document-extract-pdf-render")]
const OCR_PENDING_RESOURCE_ARROW_NAME: &str = "_ocr_pending.arrow";
#[cfg(feature = "document-extract-pdf-render")]
pub(crate) const OCR2_REGION_SCAFFOLD_FILE_NAME: &str = "_hosted_vlm_region_scaffolds.json";

pub(crate) async fn materialize_ocr2_recovery_page_images(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let recovery_pages = inputs
        .iter()
        .filter(|input| {
            input.shard_type == "page"
                && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
                && !Path::new(input.image_path.as_str()).is_file()
        })
        .map(|input| input.page_index)
        .collect::<BTreeSet<_>>();
    if recovery_pages.is_empty() {
        return Ok(inputs);
    }

    #[cfg(feature = "document-extract-pdf-render")]
    {
        let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
        let output_dir = Path::new(render_report.output_dir.as_str()).join("_ocr2-page-renders");
        let page_indices = recovery_pages.iter().copied().collect::<Vec<_>>();
        let render_profile =
            hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
        let page_render_report = tokio::task::spawn_blocking(move || {
            render_pdf_page_shards_for_page_indices(
                source_path.as_path(),
                output_dir.as_path(),
                &render_profile,
                page_indices.as_slice(),
            )
        })
        .await
        .map_err(|error| format!("join hosted VLM/OCR recovery page render task: {error}"))??;
        let ocr_input_path = hybrid_page_ocr_input_arrow_path(&page_render_report)?;
        let input_batches = read_arrow_file(ocr_input_path.as_path())?;
        let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
        merge_ocr2_recovery_page_inputs(inputs, rendered_inputs)
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        Err(
            "hosted VLM/OCR recovery pages require the `document-extract-pdf-render` feature"
                .to_string(),
        )
    }
}

pub(crate) async fn materialize_ocr2_recovery_region_images(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Ocr2RegionMaterialization, String> {
    #[cfg(feature = "document-extract-pdf-render")]
    {
        materialize_ocr2_recovery_region_images_render(render_report, inputs).await
    }

    #[cfg(not(feature = "document-extract-pdf-render"))]
    {
        let _ = render_report;
        let _ = inputs;
        Err(
            "hosted VLM/OCR recovery regions require the `document-extract-pdf-render` feature"
                .to_string(),
        )
    }
}

#[cfg(feature = "document-extract-pdf-render")]
async fn materialize_ocr2_recovery_region_images_render(
    render_report: &PdfPageRenderShardReport,
    inputs: Vec<PdfOcrShardInput>,
) -> Result<Ocr2RegionMaterialization, String> {
    let source_path = Path::new(render_report.source_path.as_str()).to_path_buf();
    let mut materialization = Ocr2RegionMaterialization::new(inputs);
    let phase_started = Instant::now();
    let (explicit_regions, regions) = ocr2_recovery_region_requests_for_inputs(
        source_path.as_path(),
        materialization.inputs.as_slice(),
    )?;
    materialization.stats.requested_region_count = regions.len();
    materialization.record_phase_elapsed("regionMaterializePlan", phase_started);
    if regions.is_empty() {
        return Ok(materialization);
    }

    let phase_started = Instant::now();
    let request_count = regions.len();
    let region_pages = regions
        .iter()
        .map(|region| region.page_index)
        .collect::<BTreeSet<_>>();
    let render_profile =
        hybrid_page_ocr_render_profile_with_lookup(true, &|key| std::env::var(key).ok());
    let source_content_hash = sha256_file_hex(source_path.as_path())?;
    let output_dir = ocr2_region_render_cache_dir_with_source_hash(
        source_content_hash.as_str(),
        &render_profile,
        regions.as_slice(),
    )?;
    let cached_region_render_report = cached_ocr2_region_render_report(
        source_path.as_path(),
        output_dir.as_path(),
        render_report.page_count,
        &render_profile,
        request_count,
    );
    let render_cache_hit = cached_region_render_report.is_some();
    let region_render_report = if let Some(report) = cached_region_render_report {
        report
    } else {
        let source_for_render = source_path.clone();
        let output_for_render = output_dir.clone();
        let regions_for_render = regions.clone();
        tokio::task::spawn_blocking(move || {
            render_pdf_region_shards_with_source_hash(PdfRegionShardRenderRequest {
                path: source_for_render.as_path(),
                output_dir: output_for_render.as_path(),
                profile: &render_profile,
                regions: regions_for_render.as_slice(),
                source_hash: source_content_hash.as_str(),
            })
        })
        .await
        .map_err(|error| format!("join hosted VLM/OCR recovery region render task: {error}"))??
    };
    materialization.record_phase_elapsed("regionMaterializeRender", phase_started);
    materialization.stats.render_reported_elapsed_ms = region_render_report.elapsed_ms;
    materialization
        .stats
        .record_render_artifact_cache_report(&region_render_report);

    let phase_started = Instant::now();
    let ocr_input_path = hybrid_page_ocr_input_arrow_path(&region_render_report)?;
    let input_batches = read_arrow_file(ocr_input_path.as_path())?;
    let rendered_inputs = decode_ocr_shard_input_batches(&input_batches)?;
    materialization.stats.rendered_region_count = rendered_inputs.len();
    if render_cache_hit {
        materialization.stats.render_cache_hit_count = rendered_inputs.len();
    } else {
        materialization.stats.render_cache_miss_count = rendered_inputs.len();
    }
    let existing_inputs = std::mem::take(&mut materialization.inputs);
    let merged_inputs =
        merge_hosted_vlm_recovery_region_inputs(existing_inputs, rendered_inputs, &region_pages)?;
    materialization.record_phase_elapsed("regionMaterializeMerge", phase_started);

    let phase_started = Instant::now();
    write_ocr2_region_scaffold_sidecar_with_lookup(
        source_path.as_path(),
        output_dir.as_path(),
        merged_inputs.as_slice(),
        explicit_regions,
        &|key| std::env::var(key).ok(),
    )?;
    materialization.inputs = merged_inputs;
    materialization.record_phase_elapsed("regionMaterializeScaffold", phase_started);
    Ok(materialization)
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_recovery_region_requests_for_inputs(
    source_path: &Path,
    inputs: &[PdfOcrShardInput],
) -> Result<(bool, Vec<PdfPageRegionRenderRequest>), String> {
    let explicit_regions = std::env::var(DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).is_ok();
    if explicit_regions {
        return hybrid_page_ocr_region_requests_for_source_with_lookup(source_path, &|key| {
            std::env::var(key).ok()
        })
        .map(|regions| (explicit_regions, regions));
    }
    if !has_ocr2_recovery_page_candidates(inputs) {
        return Ok((explicit_regions, Vec::new()));
    }
    Ok((
        explicit_regions,
        automatic_ocr2_recovery_region_requests_for_source_with_lookup(
            source_path,
            inputs,
            &|key| std::env::var(key).ok(),
        ),
    ))
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_region_render_cache_dir_with_source_hash(
    source_content_hash: &str,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<PathBuf, String> {
    Ok(
        ocr2_region_render_cache_root().join(ocr2_region_render_cache_key_with_source_hash(
            source_content_hash,
            profile,
            regions,
        )?),
    )
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_region_render_cache_root() -> PathBuf {
    if let Some(root) = std::env::var_os(DOCUMENT_EXTRACT_OCR_SHARD_CACHE_ROOT_ENV) {
        let root = PathBuf::from(root);
        return root.parent().map_or_else(
            || root.join(OCR2_REGION_RENDER_CACHE_DIR_NAME),
            |parent| parent.join(OCR2_REGION_RENDER_CACHE_DIR_NAME),
        );
    }
    let cache_root =
        std::env::var_os("PRJ_CACHE_HOME").map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
    cache_root
        .join("wendao-document-extract")
        .join(OCR2_REGION_RENDER_CACHE_DIR_NAME)
}

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) fn ocr2_region_render_cache_key(
    source: &Path,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<String, String> {
    let source_content_hash = sha256_file_hex(source)?;
    ocr2_region_render_cache_key_with_source_hash(source_content_hash.as_str(), profile, regions)
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_region_render_cache_key_with_source_hash(
    source_content_hash: &str,
    profile: &PdfPageRenderProfile,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<String, String> {
    let payload = serde_json::to_vec(&json!({
        "schema": "xiuxian_wendao.ocr2_region_render_cache_key.v1",
        "sourceContentHash": source_content_hash,
        "renderProfile": profile,
        "regions": regions,
    }))
    .map_err(|error| format!("serialize hosted VLM/OCR region render cache key: {error}"))?;
    Ok(sha256_hex(payload.as_slice()))
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn cached_ocr2_region_render_report(
    source: &Path,
    output_dir: &Path,
    page_count: u32,
    profile: &PdfPageRenderProfile,
    request_count: usize,
) -> Option<PdfPageRenderShardReport> {
    let manifest_arrow_path = output_dir.join(OCR_SHARD_MANIFEST_ARROW_NAME);
    let ocr_input_arrow_path = output_dir.join(OCR_SHARD_INPUT_ARROW_NAME);
    let pending_resource_arrow_path = output_dir.join(OCR_PENDING_RESOURCE_ARROW_NAME);
    if !manifest_arrow_path.is_file()
        || !ocr_input_arrow_path.is_file()
        || !pending_resource_arrow_path.is_file()
    {
        return None;
    }

    let Ok(input_batches) = read_arrow_file(ocr_input_arrow_path.as_path()) else {
        return None;
    };
    let Ok(inputs) = decode_ocr_shard_input_batches(&input_batches) else {
        return None;
    };
    if inputs.len() != request_count
        || inputs.iter().any(|input| {
            input.shard_type != "region" || !Path::new(input.image_path.as_str()).is_file()
        })
    {
        return None;
    }

    Some(PdfPageRenderShardReport {
        source_path: source.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        page_count,
        shard_count: u32::try_from(inputs.len()).unwrap_or(u32::MAX),
        manifest_arrow_path: Some(manifest_arrow_path.to_string_lossy().to_string()),
        ocr_input_arrow_path: Some(ocr_input_arrow_path.to_string_lossy().to_string()),
        pending_resource_arrow_path: Some(
            pending_resource_arrow_path.to_string_lossy().to_string(),
        ),
        render_profile: profile.profile_id.clone(),
        render_selection: "region_shards".to_string(),
        status: PdfRenderStatus::Rendered.as_str().to_string(),
        routing_decision: PdfRenderRoutingDecision::HybridPageOcrCandidate
            .as_str()
            .to_string(),
        elapsed_ms: 0.0,
        error_message: None,
        artifact_cache_backend: None,
        artifact_cache_hit_count: 0,
        artifact_cache_miss_count: 0,
        artifact_cache_throttled_count: 0,
        artifact_cache_byte_count: 0,
        artifact_cache_page_raster_hit_count: 0,
        artifact_cache_page_raster_miss_count: 0,
        artifact_cache_page_raster_throttled_count: 0,
        artifact_cache_page_raster_byte_count: 0,
        artifact_cache_region_crop_hit_count: 0,
        artifact_cache_region_crop_miss_count: 0,
        artifact_cache_region_crop_throttled_count: 0,
        artifact_cache_region_crop_byte_count: 0,
        artifact_cache_region_manifest_projection_hit_count: 0,
        artifact_cache_region_manifest_projection_miss_count: 0,
        artifact_cache_region_manifest_projection_throttled_count: 0,
        artifact_cache_region_manifest_projection_byte_count: 0,
        artifact_cache_region_manifest_projection_row_hit_count: 0,
        artifact_cache_region_manifest_projection_row_miss_count: 0,
        artifact_cache_region_manifest_projection_row_throttled_count: 0,
        artifact_cache_region_manifest_projection_row_byte_count: 0,
    })
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn has_ocr2_recovery_page_candidates(inputs: &[PdfOcrShardInput]) -> bool {
    inputs.iter().any(|input| {
        input.shard_type == "page" && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
    })
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn write_ocr2_region_scaffold_sidecar_with_lookup(
    source: &Path,
    output_dir: &Path,
    inputs: &[PdfOcrShardInput],
    explicit_regions: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<(), String> {
    let Some(payload) = ocr2_region_scaffold_payload(source, inputs, explicit_regions, lookup)
    else {
        return Ok(());
    };
    std::fs::create_dir_all(output_dir)
        .map_err(|error| format!("create hosted VLM/OCR scaffold output directory: {error}"))?;
    let bytes = serde_json::to_vec_pretty(&payload)
        .map_err(|error| format!("serialize hosted VLM/OCR region scaffold sidecar: {error}"))?;
    std::fs::write(output_dir.join(OCR2_REGION_SCAFFOLD_FILE_NAME), bytes)
        .map_err(|error| format!("write hosted VLM/OCR region scaffold sidecar: {error}"))
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| {
        format!(
            "read hosted VLM/OCR region render cache source `{}`: {error}",
            path.display()
        )
    })?;
    Ok(sha256_hex(bytes.as_slice()))
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_region_scaffold_payload(
    source: &Path,
    inputs: &[PdfOcrShardInput],
    explicit_regions: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<Value> {
    if hybrid_page_ocr2_scaffold_mode_with_lookup(lookup)
        != HybridPdfOcr2ScaffoldMode::RegionTableJson
    {
        return None;
    }
    let region_inputs = inputs
        .iter()
        .filter(|input| {
            input.shard_type == "region" && is_hosted_vlm_direct_profile(input.ocr_profile.as_str())
        })
        .collect::<Vec<_>>();
    if region_inputs.is_empty() {
        return None;
    }

    let profiles = source_pdf_page_profiles_cached(source).unwrap_or_default();
    let profiles_by_page = profiles
        .iter()
        .map(|profile| (profile.page_index, profile))
        .collect::<BTreeMap<_, _>>();
    let items = region_inputs
        .iter()
        .map(|input| {
            let profile = profiles_by_page.get(&input.page_index).copied();
            json!({
                "scaffoldKind": ocr2_region_scaffold_kind(profile, explicit_regions),
                "shardElementId": input.shard_element_id,
                "parentShardElementId": input.parent_shard_element_id,
                "pageIndex": input.page_index,
                "regionIndex": input.region_index,
                "sourcePath": input.source_path,
                "sourceContentHash": input.source_content_hash,
                "rasterSha256": input.raster_sha256,
                "renderDpi": input.render_dpi,
                "imagePath": input.image_path,
                "cropBox": {
                    "left": input.crop_left,
                    "bottom": input.crop_bottom,
                    "right": input.crop_right,
                    "top": input.crop_top,
                },
                "sourcePagePixelBox": {
                    "left": input.source_page_pixel_left,
                    "top": input.source_page_pixel_top,
                    "right": input.source_page_pixel_right,
                    "bottom": input.source_page_pixel_bottom,
                },
                "sourcePageProfile": profile.map(source_page_profile_json),
            })
        })
        .collect::<Vec<_>>();
    Some(json!({
        "schema": "xiuxian_wendao.hosted_vlm_region_scaffold.v1",
        "mode": "region-table-json",
        "sourcePath": source.to_string_lossy(),
        "items": items,
    }))
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn ocr2_region_scaffold_kind(
    profile: Option<&PdfSourcePageProfile>,
    explicit_regions: bool,
) -> &'static str {
    if explicit_regions {
        return "manual_region_candidate";
    }
    let Some(profile) = profile else {
        return "complex_layout_candidate";
    };
    if profile.rectangle_ops > 0 || profile.path_ops >= 64 {
        "table_candidate"
    } else {
        "complex_layout_candidate"
    }
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn source_page_profile_json(profile: &PdfSourcePageProfile) -> Value {
    json!({
        "pageIndex": profile.page_index,
        "contentBytes": profile.content_bytes,
        "operationCount": profile.operation_count,
        "textShowOps": profile.text_show_ops,
        "pathOps": profile.path_ops,
        "rectangleOps": profile.rectangle_ops,
        "drawObjectOps": profile.draw_object_ops,
        "estimatedWeight": profile.estimated_weight,
    })
}

#[cfg(feature = "document-extract-pdf-render")]
pub(crate) fn merge_ocr2_recovery_page_inputs(
    mut inputs: Vec<PdfOcrShardInput>,
    rendered_inputs: Vec<PdfOcrShardInput>,
) -> Result<Vec<PdfOcrShardInput>, String> {
    let rendered_by_page = rendered_inputs
        .into_iter()
        .map(|input| (input.page_index, input))
        .collect::<BTreeMap<u32, PdfOcrShardInput>>();
    for input in &mut inputs {
        if input.shard_type != "page" || !is_hosted_vlm_direct_profile(input.ocr_profile.as_str()) {
            continue;
        }
        let Some(rendered) = rendered_by_page.get(&input.page_index) else {
            return Err(format!(
                "hosted VLM/OCR recovery render did not produce page {}",
                input.page_index
            ));
        };
        let ocr_profile = input.ocr_profile.clone();
        let ocr_engine = input.ocr_engine.clone();
        *input = rendered.clone();
        input.ocr_profile = ocr_profile;
        input.ocr_engine = ocr_engine;
    }
    Ok(inputs)
}
