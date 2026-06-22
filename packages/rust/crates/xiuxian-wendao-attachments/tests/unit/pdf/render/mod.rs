use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;
use std::path::{Path, PathBuf};

use crate::pdf::render::{
    PdfPageBox, PdfPageRegion, PdfPageRegionRenderRequest, PdfPageRegionShardManifestInput,
    PdfPageRenderProfile, PdfPageShardManifest, PdfPageShardManifestInput, RenderedRasterIdentity,
    build_region_shard_manifest, build_shard_manifest, page_region_render_request_chunks_by_page,
    page_region_render_request_chunks_by_page_area_desc,
    page_region_render_request_chunks_by_page_max_area_desc,
    page_region_render_request_chunks_by_region,
};

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {actual} to be close to {expected}"
    );
}

fn int32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("`{name}` column is not Int32"))
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("`{name}` column is not Float64"))
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

fn sample_manifest(rotation_degrees: u16) -> PdfPageShardManifest {
    let profile = PdfPageRenderProfile::ocr_default();
    build_shard_manifest(PdfPageShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 2,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        rotation_degrees,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00002.png"),
            sha256: "rasterhash".to_string(),
            width_px: 2400,
            height_px: 3100,
        },
    })
}

fn sample_region_manifest() -> Result<PdfPageShardManifest, String> {
    let page_manifest = sample_manifest(0);
    let profile = PdfPageRenderProfile::ocr_default();
    build_region_shard_manifest(PdfPageRegionShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 2,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        page_crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        region: PdfPageRegion::new(
            7,
            PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
            page_manifest.element_id,
            "000002.000007",
        ),
        rotation_degrees: 0,
        page_raster_width_px: 2400,
        page_raster_height_px: 3100,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00002-region-00007.png"),
            sha256: "regionhash".to_string(),
            width_px: 600,
            height_px: 775,
        },
    })
}

mod arrows;
mod artifacts;
mod geometry;

#[test]
fn page_region_render_request_chunks_group_by_page_and_reading_order() {
    let chunks = page_region_render_request_chunks_by_page(&[
        PdfPageRegionRenderRequest::new(
            2,
            2,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000001".to_string()),
        ),
    ]);

    assert_eq!(chunks.len(), 2);
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|region| (region.page_index, region.region_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(1, 1)], vec![(2, 1), (2, 2)]]
    );
}

#[test]
fn page_region_render_request_chunks_by_region_preserve_reading_order() {
    let chunks = page_region_render_request_chunks_by_region(&[
        PdfPageRegionRenderRequest::new(
            2,
            2,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 1.0, 1.0),
            Some("000002.000001".to_string()),
        ),
    ]);

    assert_eq!(chunks.len(), 3);
    assert_eq!(
        chunks
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|region| (region.page_index, region.region_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(1, 1)], vec![(2, 1)], vec![(2, 2)]]
    );
}

#[test]
fn page_region_render_request_chunks_by_page_area_desc_prioritize_large_pages() {
    let chunks = page_region_render_request_chunks_by_page_area_desc(&[
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 10.0, 10.0),
            Some("000002.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            2,
            PdfPageBox::new(0.0, 0.0, 15.0, 15.0),
            Some("000001.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 15.0, 15.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            3,
            1,
            PdfPageBox::new(0.0, 0.0, 8.0, 8.0),
            Some("000003.000001".to_string()),
        ),
    ]);

    assert_eq!(
        chunks
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|region| (region.page_index, region.region_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(1, 1), (1, 2)], vec![(2, 1)], vec![(3, 1)]]
    );
}

#[test]
fn page_region_render_request_chunks_by_page_max_area_desc_prioritize_largest_region() {
    let chunks = page_region_render_request_chunks_by_page_max_area_desc(&[
        PdfPageRegionRenderRequest::new(
            2,
            1,
            PdfPageBox::new(0.0, 0.0, 30.0, 30.0),
            Some("000002.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            2,
            PdfPageBox::new(0.0, 0.0, 25.0, 25.0),
            Some("000001.000002".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            1,
            1,
            PdfPageBox::new(0.0, 0.0, 25.0, 25.0),
            Some("000001.000001".to_string()),
        ),
        PdfPageRegionRenderRequest::new(
            3,
            1,
            PdfPageBox::new(0.0, 0.0, 8.0, 8.0),
            Some("000003.000001".to_string()),
        ),
    ]);

    assert_eq!(
        chunks
            .iter()
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|region| (region.page_index, region.region_index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![vec![(2, 1)], vec![(1, 1), (1, 2)], vec![(3, 1)]]
    );
}
