use std::path::{Path, PathBuf};

use arrow::array::{Array, BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;

use crate::pdf::ocr::{
    PDF_OCR_SHARD_INPUT_SCHEMA_VERSION, PDF_OCR_SHARD_RESULT_SCHEMA_VERSION, PdfOcrShardResult,
    PdfOcrWorkerProfile, build_ocr_result_resource_batch, build_ocr_shard_input_batch,
    build_ocr_shard_inputs, build_ocr_shard_result_batch, decode_ocr_shard_input_batch,
    decode_ocr_shard_result_batch,
};
use crate::pdf::render::{
    PdfPageBox, PdfPageRegion, PdfPageRegionShardManifestInput, PdfPageRenderProfile,
    PdfPageShardManifest, PdfPageShardManifestInput, RenderedRasterIdentity,
    build_region_shard_manifest, build_shard_manifest,
};

mod input;
mod recovery;
mod result;

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

fn bool_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a BooleanArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("`{name}` column is not Boolean"))
}

fn sample_manifest() -> PdfPageShardManifest {
    let profile = PdfPageRenderProfile::ocr_default();
    build_shard_manifest(PdfPageShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 3,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        rotation_degrees: 90,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00003.png"),
            sha256: "rasterhash".to_string(),
            width_px: 3100,
            height_px: 2400,
        },
    })
}

fn sample_region_manifest() -> Result<PdfPageShardManifest, String> {
    let page_manifest = sample_manifest();
    let profile = PdfPageRenderProfile::ocr_default();
    build_region_shard_manifest(PdfPageRegionShardManifestInput {
        source_path: Path::new("/tmp/source.pdf"),
        source_content_hash: "sourcehash",
        page_index: 3,
        profile: &profile,
        media_box: PdfPageBox::new(0.0, 0.0, 612.0, 792.0),
        page_crop_box: PdfPageBox::new(18.0, 24.0, 594.0, 768.0),
        region: PdfPageRegion::new(
            4,
            PdfPageBox::new(162.0, 210.0, 306.0, 396.0),
            page_manifest.element_id,
            "000003.000004",
        ),
        rotation_degrees: 90,
        page_raster_width_px: 3100,
        page_raster_height_px: 2400,
        raster: RenderedRasterIdentity {
            path: PathBuf::from("/tmp/shards/page-00003-region-00004.png"),
            sha256: "regionhash".to_string(),
            width_px: 620,
            height_px: 800,
        },
    })
}
