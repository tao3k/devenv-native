//! PDF page and region raster shard planning, rendering, and Arrow sidecars.

#[cfg(feature = "pdf-render")]
mod artifact_cache;
mod batches;
mod document;
mod entrypoints;
mod identity;
mod manifest;
mod regions;
mod report;
mod selection;
mod types;

pub use batches::{build_ocr_pending_resource_batch, build_shard_manifest_batch};
#[cfg(feature = "pdf-render")]
pub use document::PDFIUM_LIBRARY_PATH_ENV;
#[cfg(feature = "pdf-render")]
pub use entrypoints::{
    PdfRegionShardRenderRequest, render_pdf_page_shards, render_pdf_page_shards_for_page_indices,
    render_pdf_page_shards_with_selection, render_pdf_region_shards,
    render_pdf_region_shards_with_source_hash,
};
pub use entrypoints::{
    prepare_pdf_source_page_range_ocr_shards_with_selection, read_render_paths_from_json,
};
pub use manifest::{
    build_region_shard_manifest, build_shard_manifest, region_pixel_box_for_crop,
    render_dimensions_for_box,
};
pub use regions::{
    page_region_render_request_chunks_all, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region,
    page_region_render_request_chunks_by_region_seed_page,
};
pub use report::write_page_render_shard_reports;
pub use selection::source_pdf_page_count;
pub use types::{
    PdfOcrShardType, PdfPageBox, PdfPagePixelBox, PdfPageRegion, PdfPageRegionRenderRequest,
    PdfPageRegionShardManifestInput, PdfPageRenderProfile, PdfPageRenderSelection,
    PdfPageRenderShardReport, PdfPageShardGeometry, PdfPageShardManifest,
    PdfPageShardManifestInput, PdfRenderRoutingDecision, PdfRenderStatus, RenderedRasterIdentity,
};

#[cfg(all(test, feature = "pdf-render"))]
#[path = "../../../tests/unit/pdf/render/mod.rs"]
mod tests;
