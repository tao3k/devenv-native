//! Artifact-cache helpers for rendered PDF raster bytes.

mod runtime;
mod xiuxian_db_store_key;

pub(super) use runtime::{
    PdfRenderArtifactCache, PdfRenderArtifactCacheStats, RegionCropArtifactIdentity,
    materialize_page_raster_identity, materialize_region_crop_identity,
    materialize_requested_region_crop_identity, pdf_render_artifact_cache_from_environment,
    save_image_bytes,
};
#[cfg(feature = "foyer-artifact-cache")]
pub(super) use runtime::{
    restore_region_manifest_projection, restore_region_manifest_projection_row,
    restore_requested_region_crop_identity, write_region_manifest_projection,
    write_region_manifest_projection_row,
};
