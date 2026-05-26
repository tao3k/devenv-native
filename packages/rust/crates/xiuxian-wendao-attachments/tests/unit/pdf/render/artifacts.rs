#[cfg(feature = "foyer-artifact-cache")]
use std::fs;
use std::path::Path;

use super::sample_region_manifest;
#[cfg(feature = "foyer-artifact-cache")]
use crate::pdf::render::artifact_cache::{
    PdfRenderArtifactCache, materialize_page_raster_identity,
    materialize_requested_region_crop_identity, restore_region_manifest_projection,
    restore_requested_region_crop_identity, write_region_manifest_projection,
    write_region_manifest_projection_row,
};
use crate::pdf::render::batches::write_shard_artifact_batches;
#[cfg(feature = "foyer-artifact-cache")]
use crate::pdf::render::document::restore_document_region_manifests_from_cache;
#[cfg(feature = "foyer-artifact-cache")]
use crate::pdf::render::report::RenderShardContext;
use crate::pdf::render::selection::{RenderPageSelection, resolve_page_selection};
#[cfg(feature = "foyer-artifact-cache")]
use crate::pdf::render::{PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile};
use crate::pdf::render::{PdfPageRenderSelection, build_shard_manifest_batch};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCacheBackend, FoyerArtifactBlobCache, FoyerArtifactBlobCacheConfig,
};

#[test]
fn document_extract_pdf_render_writes_region_arrow_artifacts() -> Result<(), String> {
    let manifest = sample_region_manifest()?;
    let manifest_batch = build_shard_manifest_batch(std::slice::from_ref(&manifest))?;
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;

    let (manifest_path, input_path, pending_path) = write_shard_artifact_batches(
        temp_dir.path(),
        std::slice::from_ref(&manifest),
        manifest_batch,
    )?;

    assert!(manifest_path.is_file());
    assert!(input_path.is_file());
    assert!(pending_path.is_file());
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
#[test]
fn document_extract_pdf_render_restores_page_raster_from_artifact_cache() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = foyer_render_cache(temp_dir.path().join("artifacts").as_path())?;
    let image_path = temp_dir.path().join("page-00000.png");
    let profile = PdfPageRenderProfile::ocr_default();
    let mut build_count = 0usize;

    let first = materialize_page_raster_identity(
        Some(&cache),
        "sourcehash",
        &profile,
        0,
        image_path.as_path(),
        || {
            build_count = build_count.saturating_add(1);
            let image = image::DynamicImage::new_rgba8(2, 3);
            image
                .save(image_path.as_path())
                .map_err(|error| format!("write test PNG: {error}"))?;
            fs::read(image_path.as_path()).map_err(|error| format!("read test PNG: {error}"))
        },
    )?;
    fs::remove_file(image_path.as_path()).map_err(|error| error.to_string())?;
    let second = materialize_page_raster_identity(
        Some(&cache),
        "sourcehash",
        &profile,
        0,
        image_path.as_path(),
        || Err("cache hit should not rebuild the raster".to_string()),
    )?;

    assert_eq!(build_count, 1);
    assert_eq!(first.sha256, second.sha256);
    assert_eq!((second.width_px, second.height_px), (2, 3));
    assert!(image_path.is_file());
    let stats = cache.snapshot();
    assert_eq!(stats.backend_name.as_deref(), Some("foyer"));
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.miss_count, 1);
    assert_eq!(stats.throttled_count, 0);
    assert!(stats.byte_count > 0);
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
#[test]
fn document_extract_pdf_render_restores_region_manifest_projection_from_artifact_cache()
-> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let cache = foyer_render_cache(temp_dir.path().join("artifacts").as_path())?;
    let profile = PdfPageRenderProfile::ocr_default();
    let request = PdfPageRegionRenderRequest::new(
        2,
        7,
        PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
        Some("000002.000007".to_string()),
    );
    let manifest = sample_region_manifest()?;
    let batch = build_shard_manifest_batch(std::slice::from_ref(&manifest))?;

    assert!(
        restore_region_manifest_projection(
            Some(&cache),
            "sourcehash",
            &profile,
            2,
            std::slice::from_ref(&request),
        )?
        .is_none()
    );
    write_region_manifest_projection(
        Some(&cache),
        "sourcehash",
        &profile,
        2,
        std::slice::from_ref(&request),
        std::slice::from_ref(&batch),
    )?;
    write_region_manifest_projection_row(
        Some(&cache),
        "sourcehash",
        &profile,
        &request,
        std::slice::from_ref(&batch),
    )?;
    let restored = restore_region_manifest_projection(
        Some(&cache),
        "sourcehash",
        &profile,
        2,
        std::slice::from_ref(&request),
    )?
    .ok_or_else(|| "expected region manifest projection artifact hit".to_string())?;

    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].num_rows(), 1);
    let stats = cache.snapshot();
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.miss_count, 1);
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
#[test]
fn document_extract_pdf_render_restores_region_projection_and_crop_from_foyer() -> Result<(), String>
{
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let artifact_root = temp_dir.path().join("artifacts");
    let cache = foyer_render_cache(artifact_root.as_path())?;
    let profile = PdfPageRenderProfile::ocr_default();
    let request = PdfPageRegionRenderRequest::new(
        2,
        7,
        PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
        Some("000002.000007".to_string()),
    );
    let image_path = temp_dir.path().join("page-00002-region-00007.png");
    let mut manifest = sample_region_manifest()?;
    let raster = materialize_requested_region_crop_identity(
        Some(&cache),
        "sourcehash",
        &profile,
        &request,
        manifest.geometry.crop_box,
        image_path.as_path(),
        || {
            let image = image::DynamicImage::new_rgba8(
                manifest.geometry.raster_width_px,
                manifest.geometry.raster_height_px,
            );
            image
                .save(image_path.as_path())
                .map_err(|error| format!("write test region crop: {error}"))?;
            fs::read(image_path.as_path())
                .map_err(|error| format!("read test region crop: {error}"))
        },
    )?;
    manifest.raster_sha256 = raster.sha256.clone();
    manifest.image_path = image_path.to_string_lossy().to_string();
    let batch = build_shard_manifest_batch(std::slice::from_ref(&manifest))?;
    write_region_manifest_projection(
        Some(&cache),
        "sourcehash",
        &profile,
        2,
        std::slice::from_ref(&request),
        std::slice::from_ref(&batch),
    )?;
    write_region_manifest_projection_row(
        Some(&cache),
        "sourcehash",
        &profile,
        &request,
        std::slice::from_ref(&batch),
    )?;
    fs::remove_file(image_path.as_path()).map_err(|error| error.to_string())?;

    let restored_projection = restore_region_manifest_projection(
        Some(&cache),
        "sourcehash",
        &profile,
        2,
        std::slice::from_ref(&request),
    )?
    .ok_or_else(|| "expected Foyer projection hit".to_string())?;
    let restored_crop = restore_requested_region_crop_identity(
        Some(&cache),
        "sourcehash",
        &profile,
        &request,
        manifest.geometry.crop_box,
        image_path.as_path(),
    )?
    .ok_or_else(|| "expected Foyer crop hit".to_string())?;

    assert_eq!(restored_projection.len(), 1);
    assert_eq!(restored_projection[0].num_rows(), 1);
    assert_eq!(restored_crop.sha256, raster.sha256);
    assert_eq!(restored_crop.width_px, manifest.geometry.raster_width_px);
    assert_eq!(restored_crop.height_px, manifest.geometry.raster_height_px);
    assert!(image_path.is_file());
    let stats_after_direct_restore = cache.snapshot();
    assert_eq!(
        stats_after_direct_restore.backend_name.as_deref(),
        Some("foyer")
    );
    assert_eq!(stats_after_direct_restore.hit_count, 2);
    assert_eq!(stats_after_direct_restore.miss_count, 1);
    assert!(stats_after_direct_restore.byte_count > 0);

    let restore_dir = temp_dir.path().join("restored-region-manifests");
    let context = RenderShardContext::new(
        Path::new("/tmp/source.pdf"),
        restore_dir.as_path(),
        &profile,
        PdfPageRenderSelection::RegionShards,
    );
    let restored_manifests = restore_document_region_manifests_from_cache(
        &context,
        "sourcehash",
        std::slice::from_ref(&request),
        Some(&cache),
    )?
    .ok_or_else(|| "expected full region manifest restore from Foyer".to_string())?;

    assert_eq!(restored_manifests.len(), 1);
    assert_eq!(restored_manifests[0].raster_sha256, raster.sha256);
    let restored_image_path = restored_manifests[0].image_path.clone();
    assert!(
        Path::new(restored_image_path.as_str()).is_file(),
        "restored crop path should be materialized from Foyer"
    );
    let stats = cache.snapshot();
    assert_eq!(stats.backend_name.as_deref(), Some("foyer"));
    assert_eq!(stats.hit_count, 4);
    assert_eq!(stats.miss_count, 1);
    assert!(stats.byte_count >= stats_after_direct_restore.byte_count);
    drop(restored_manifests);
    drop(cache);

    fs::remove_file(restored_image_path.as_str()).map_err(|error| error.to_string())?;
    let reopened_cache = foyer_render_cache(artifact_root.as_path())?;
    let reopened_manifests = restore_document_region_manifests_from_cache(
        &context,
        "sourcehash",
        std::slice::from_ref(&request),
        Some(&reopened_cache),
    )?
    .ok_or_else(|| "expected restart region manifest restore from Foyer".to_string())?;
    assert_eq!(reopened_manifests.len(), 1);
    assert_eq!(reopened_manifests[0].raster_sha256, raster.sha256);
    assert!(
        Path::new(reopened_manifests[0].image_path.as_str()).is_file(),
        "reopened Foyer cache should restore the crop path"
    );
    let reopened_stats = reopened_cache.snapshot();
    assert_eq!(reopened_stats.backend_name.as_deref(), Some("foyer"));
    assert_eq!(reopened_stats.hit_count, 2);
    assert_eq!(reopened_stats.miss_count, 0);
    assert!(reopened_stats.byte_count > 0);
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
fn foyer_render_cache(root: &Path) -> Result<PdfRenderArtifactCache, String> {
    let backend = FoyerArtifactBlobCache::from_config(FoyerArtifactBlobCacheConfig::new(
        root,
        4 * 1024 * 1024,
        16 * 1024 * 1024,
    ))
    .map(ArtifactBlobCacheBackend::Foyer)
    .map_err(|error| format!("build test Foyer artifact cache: {error}"))?;
    Ok(PdfRenderArtifactCache::from_backend(backend))
}

#[test]
fn document_extract_pdf_render_shard_fallback_defaults_to_all_pages_without_detector()
-> Result<(), String> {
    let selection = resolve_page_selection(
        Path::new("fixture.pdf"),
        PdfPageRenderSelection::ShardFallbackPages,
    )?;

    assert!(matches!(selection, RenderPageSelection::All));
    Ok(())
}
