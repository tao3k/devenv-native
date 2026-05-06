//! PDF page and region raster shard planning, rendering, and Arrow sidecars.

mod batches;
mod document;
mod entrypoints;
mod identity;
mod manifest;
mod report;
mod selection;
mod types;

pub use batches::{build_ocr_pending_resource_batch, build_shard_manifest_batch};
#[cfg(feature = "pdf-render")]
pub use document::PDFIUM_LIBRARY_PATH_ENV;
pub use entrypoints::{
    prepare_pdf_source_page_range_ocr_shards_with_selection, read_render_paths_from_json,
};
#[cfg(feature = "pdf-render")]
pub use entrypoints::{
    render_pdf_page_shards, render_pdf_page_shards_for_page_indices,
    render_pdf_page_shards_with_selection, render_pdf_region_shards,
};
pub use manifest::{
    build_region_shard_manifest, build_shard_manifest, region_pixel_box_for_crop,
    render_dimensions_for_box,
};
pub use report::write_page_render_shard_reports;
pub use types::{
    PdfOcrShardType, PdfPageBox, PdfPagePixelBox, PdfPageRegion, PdfPageRegionRenderRequest,
    PdfPageRegionShardManifestInput, PdfPageRenderProfile, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfPageShardGeometry, PdfPageShardManifest,
    PdfPageShardManifestInput, PdfRenderRoutingDecision, PdfRenderStatus, RenderedRasterIdentity,
};

#[cfg(all(test, feature = "pdf-render"))]
use batches::write_shard_artifact_batches;
#[cfg(all(test, feature = "pdf-render"))]
use document::save_region_crop_image;
#[cfg(all(test, feature = "pdf-render"))]
use identity::shard_element_id;
#[cfg(all(test, feature = "pdf-render"))]
use selection::{RenderPageSelection, resolve_page_selection};

#[cfg(all(test, feature = "pdf-render"))]
#[path = "../../../tests/unit/pdf/render/mod.rs"]
mod tests;
