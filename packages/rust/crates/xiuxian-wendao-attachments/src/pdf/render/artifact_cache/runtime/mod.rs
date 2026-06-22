//! Artifact-cache helpers for rendered PDF raster bytes.

mod backend;
mod keys;
mod manifest;
mod model;
mod raster;

pub(crate) use backend::pdf_render_artifact_cache_from_environment;
#[cfg(feature = "foyer-artifact-cache")]
pub(crate) use manifest::{
    restore_region_manifest_projection, restore_region_manifest_projection_row,
    write_region_manifest_projection, write_region_manifest_projection_row,
};
pub(crate) use model::{
    PdfRenderArtifactCache, PdfRenderArtifactCacheStats, RegionCropArtifactIdentity,
};
#[cfg(feature = "foyer-artifact-cache")]
pub(crate) use raster::restore_requested_region_crop_identity;
pub(crate) use raster::{
    materialize_page_raster_identity, materialize_region_crop_identity,
    materialize_requested_region_crop_identity, save_image_bytes,
};
