//! PDF render raster artifact materialization and restoration.

use std::fs;
use std::path::Path;

#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobReadStatus, ArtifactCacheError, read_through_artifact_bytes,
};

use crate::pdf::render::identity::sha256_hex;
use crate::pdf::render::types::{
    PdfPageBox, PdfPageRegionRenderRequest, PdfPageRenderProfile, RenderedRasterIdentity,
};

use super::keys::{page_raster_artifact_key, region_crop_artifact_key};
use super::model::{
    PdfRenderArtifactCache, PdfRenderArtifactRecordKind, RegionCropArtifactIdentity,
};
use crate::pdf::render::artifact_cache::xiuxian_db_store_key;

pub(crate) fn materialize_page_raster_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    materialize_raster_identity(
        cache,
        PdfRenderArtifactRecordKind::PageRaster,
        || page_raster_artifact_key(source_hash, profile, page_index),
        image_path,
        build_bytes,
    )
}

pub(crate) fn materialize_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    identity: RegionCropArtifactIdentity<'_>,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        let _ = (
            identity.source_hash,
            identity.profile,
            identity.page_index,
            identity.region_index,
            identity.region_box,
        );
    }
    materialize_raster_identity(
        cache,
        PdfRenderArtifactRecordKind::RegionCrop,
        || region_crop_artifact_key(identity),
        image_path,
        build_bytes,
    )
}

pub(crate) fn materialize_requested_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    region_box: PdfPageBox,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    materialize_region_crop_identity(
        cache,
        RegionCropArtifactIdentity {
            source_hash,
            profile,
            page_index: request.page_index,
            region_index: request.region_index,
            region_box,
        },
        image_path,
        build_bytes,
    )
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_requested_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    region_box: PdfPageBox,
    image_path: &Path,
) -> Result<Option<RenderedRasterIdentity>, String> {
    restore_region_crop_identity(
        cache,
        RegionCropArtifactIdentity {
            source_hash,
            profile,
            page_index: request.page_index,
            region_index: request.region_index,
            region_box,
        },
        image_path,
    )
}

#[cfg(feature = "foyer-artifact-cache")]
fn restore_region_crop_identity(
    cache: Option<&PdfRenderArtifactCache>,
    identity: RegionCropArtifactIdentity<'_>,
    image_path: &Path,
) -> Result<Option<RenderedRasterIdentity>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_crop_artifact_key(identity)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region crop artifact cache entry: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(PdfRenderArtifactRecordKind::RegionCrop, read.byte_len());
            write_restored_bytes(image_path, read.bytes())?;
            raster_identity_from_bytes(image_path, read.bytes()).map(Some)
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionCrop);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionCrop);
            Ok(None)
        }
    }
}

pub(crate) fn save_image_bytes(
    image: &image::DynamicImage,
    image_path: &Path,
) -> Result<Vec<u8>, String> {
    image
        .save(image_path)
        .map_err(|error| format!("write shard image `{}`: {error}", image_path.display()))?;
    fs::read(image_path)
        .map_err(|error| format!("read shard image `{}`: {error}", image_path.display()))
}

fn materialize_raster_identity(
    cache: Option<&PdfRenderArtifactCache>,
    #[cfg_attr(not(feature = "foyer-artifact-cache"), allow(unused_variables))]
    kind: PdfRenderArtifactRecordKind,
    key: impl FnOnce() -> Result<xiuxian_db_store_key::ArtifactKey, String>,
    image_path: &Path,
    build_bytes: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<RenderedRasterIdentity, String> {
    #[cfg(feature = "foyer-artifact-cache")]
    if let Some(cache) = cache {
        let key = key()?;
        let artifact = read_through_artifact_bytes(cache.backend.as_ref(), &key, || {
            build_bytes().map_err(|error| {
                ArtifactCacheError::backend("pdf-render", "materializing raster bytes", error)
            })
        })
        .map_err(|error| format!("materialize PDF render artifact cache entry: {error}"))?;
        cache.record(kind, &artifact);
        write_restored_bytes(image_path, artifact.bytes())?;
        return raster_identity_from_bytes(image_path, artifact.bytes());
    }
    #[cfg(not(feature = "foyer-artifact-cache"))]
    {
        let _ = cache;
        let _ = key;
    }

    let bytes = build_bytes()?;
    raster_identity_from_bytes(image_path, bytes.as_slice())
}

#[cfg(feature = "foyer-artifact-cache")]
fn write_restored_bytes(image_path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = image_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create PDF render artifact output directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(image_path, bytes).map_err(|error| {
        format!(
            "restore PDF render artifact `{}`: {error}",
            image_path.display()
        )
    })
}

fn raster_identity_from_bytes(
    image_path: &Path,
    bytes: &[u8],
) -> Result<RenderedRasterIdentity, String> {
    let image = image::load_from_memory(bytes).map_err(|error| {
        format!(
            "decode PDF render artifact `{}`: {error}",
            image_path.display()
        )
    })?;
    Ok(RenderedRasterIdentity {
        path: image_path.to_path_buf(),
        sha256: sha256_hex(bytes),
        width_px: image.width(),
        height_px: image.height(),
    })
}
