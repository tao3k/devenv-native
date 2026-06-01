//! PDF render region manifest projection artifact cache access.

use arrow::record_batch::RecordBatch;

#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::artifact_cache::{ArtifactBlobCache, ArtifactBlobReadStatus};
#[cfg(feature = "foyer-artifact-cache")]
use xiuxian_db_store::{decode_record_batches_ipc, write_record_batches_ipc_artifact};

use crate::pdf::render::types::{PdfPageRegionRenderRequest, PdfPageRenderProfile};

use super::keys::{
    region_manifest_projection_artifact_key, region_manifest_projection_row_artifact_key,
};
use super::model::{PdfRenderArtifactCache, PdfRenderArtifactRecordKind};

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_region_manifest_projection(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    regions: &[PdfPageRegionRenderRequest],
) -> Result<Option<Vec<RecordBatch>>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_manifest_projection_artifact_key(source_hash, profile, page_index, regions)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region manifest projection artifact: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(
                PdfRenderArtifactRecordKind::RegionManifestProjection,
                read.byte_len(),
            );
            decode_record_batches_ipc(read.bytes())
                .map(Some)
                .map_err(|error| format!("decode PDF region manifest projection: {error}"))
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionManifestProjection);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionManifestProjection);
            Ok(None)
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn restore_region_manifest_projection_row(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
) -> Result<Option<Vec<RecordBatch>>, String> {
    let Some(cache) = cache else {
        return Ok(None);
    };
    let key = region_manifest_projection_row_artifact_key(source_hash, profile, request)?;
    match cache
        .backend
        .read_with_status(&key)
        .map_err(|error| format!("read PDF region manifest row projection artifact: {error}"))?
    {
        ArtifactBlobReadStatus::Hit(read) => {
            cache.record_hit(
                PdfRenderArtifactRecordKind::RegionManifestProjectionRow,
                read.byte_len(),
            );
            decode_record_batches_ipc(read.bytes())
                .map(Some)
                .map_err(|error| format!("decode PDF region manifest row projection: {error}"))
        }
        ArtifactBlobReadStatus::Miss => {
            cache.record_miss(PdfRenderArtifactRecordKind::RegionManifestProjectionRow);
            Ok(None)
        }
        ArtifactBlobReadStatus::Throttled => {
            cache.record_throttled(PdfRenderArtifactRecordKind::RegionManifestProjectionRow);
            Ok(None)
        }
    }
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn write_region_manifest_projection(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    page_index: u32,
    regions: &[PdfPageRegionRenderRequest],
    batches: &[RecordBatch],
) -> Result<(), String> {
    if let Some(cache) = cache {
        let key =
            region_manifest_projection_artifact_key(source_hash, profile, page_index, regions)?;
        write_record_batches_ipc_artifact(cache.backend.as_ref(), &key, batches)
            .map_err(|error| format!("write PDF region manifest projection artifact: {error}"))?;
    }
    Ok(())
}

#[cfg(feature = "foyer-artifact-cache")]
pub(crate) fn write_region_manifest_projection_row(
    cache: Option<&PdfRenderArtifactCache>,
    source_hash: &str,
    profile: &PdfPageRenderProfile,
    request: &PdfPageRegionRenderRequest,
    batches: &[RecordBatch],
) -> Result<(), String> {
    if let Some(cache) = cache {
        let key = region_manifest_projection_row_artifact_key(source_hash, profile, request)?;
        write_record_batches_ipc_artifact(cache.backend.as_ref(), &key, batches).map_err(
            |error| format!("write PDF region manifest row projection artifact: {error}"),
        )?;
    }
    Ok(())
}
