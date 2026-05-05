//! Arrow batch builders and IPC sidecar writers for render shard artifacts.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;

use crate::pdf::ocr::{PdfOcrWorkerProfile, build_ocr_shard_input_batch, build_ocr_shard_inputs};

use super::types::{PdfOcrShardType, PdfPageShardManifest};

const OCR_SHARD_MANIFEST_ARROW_NAME: &str = "_ocr_shards.arrow";
const OCR_SHARD_INPUT_ARROW_NAME: &str = "_ocr_input.arrow";
const OCR_PENDING_RESOURCE_ARROW_NAME: &str = "_ocr_pending.arrow";

/// # Errors
///
/// Returns an error if Arrow cannot build a typed shard manifest batch.
pub fn build_shard_manifest_batch(
    manifests: &[PdfPageShardManifest],
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        shard_manifest_schema(),
        vec![
            string_manifest_column(manifests, |manifest| manifest.source_path.clone()),
            string_manifest_column(manifests, |manifest| manifest.source_content_hash.clone()),
            int_manifest_column(manifests, |manifest| manifest.page_index),
            string_manifest_column(manifests, |manifest| manifest.render_profile.clone()),
            string_manifest_column(manifests, |manifest| manifest.image_path.clone()),
            string_manifest_column(manifests, |manifest| manifest.image_mime_type.clone()),
            string_manifest_column(manifests, |manifest| manifest.raster_sha256.clone()),
            int_manifest_column(manifests, |manifest| manifest.geometry.raster_width_px),
            int_manifest_column(manifests, |manifest| manifest.geometry.raster_height_px),
            int_manifest_column(manifests, |manifest| manifest.geometry.render_dpi),
            Arc::new(Int32Array::from(
                manifests
                    .iter()
                    .map(|manifest| i32::from(manifest.geometry.rotation_degrees))
                    .collect::<Vec<_>>(),
            )),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.left),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.bottom),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.right),
            float_manifest_column(manifests, |manifest| manifest.geometry.media_box.top),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.left),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.bottom),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.right),
            float_manifest_column(manifests, |manifest| manifest.geometry.crop_box.top),
            float_manifest_column(manifests, |manifest| {
                manifest.geometry.point_to_pixel_scale_x
            }),
            float_manifest_column(manifests, |manifest| {
                manifest.geometry.point_to_pixel_scale_y
            }),
            string_manifest_column(manifests, |manifest| manifest.element_id.clone()),
            string_manifest_column(manifests, |manifest| {
                manifest.shard_type.as_str().to_string()
            }),
            int_manifest_column(manifests, |manifest| manifest.region_index),
            string_manifest_column(manifests, |manifest| {
                manifest.parent_shard_element_id.clone()
            }),
            string_manifest_column(manifests, |manifest| manifest.reading_order_key.clone()),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.left),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.top),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.right),
            int_manifest_column(manifests, |manifest| manifest.source_page_pixel_box.bottom),
        ],
    )
    .map_err(|error| format!("build OCR shard manifest Arrow batch: {error}"))
}

/// # Errors
///
/// Returns an error if Arrow cannot build the stable document-resource batch.
pub fn build_ocr_pending_resource_batch(
    manifests: &[PdfPageShardManifest],
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.source_path.as_str())
                    .collect::<Vec<_>>(),
            )) as ArrayRef,
            Arc::new(StringArray::from(vec!["ocr_pending"; manifests.len()])),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.image_path.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(Int32Array::from(
                manifests
                    .iter()
                    .map(|manifest| i32::try_from(manifest.page_index).unwrap_or(i32::MAX))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(pending_caption)
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| {
                        format!(
                            "manifest={},raster_sha256={},profile={},shard_type={},reading_order_key={}",
                            OCR_SHARD_MANIFEST_ARROW_NAME,
                            manifest.raster_sha256,
                            manifest.render_profile,
                            manifest.shard_type.as_str(),
                            manifest.reading_order_key
                        )
                    })
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.image_mime_type.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(vec!["pending"; manifests.len()])),
            Arc::new(StringArray::from(
                manifests
                    .iter()
                    .map(|manifest| manifest.element_id.as_str())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| format!("build OCR pending resource Arrow batch: {error}"))
}
fn pending_caption(manifest: &PdfPageShardManifest) -> String {
    match manifest.shard_type {
        PdfOcrShardType::Page => format!("OCR pending PDF page {}", manifest.page_index + 1),
        PdfOcrShardType::Region => format!(
            "OCR pending PDF page {} region {}",
            manifest.page_index + 1,
            manifest.region_index
        ),
    }
}
fn string_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> String,
{
    Arc::new(StringArray::from(
        manifests.iter().map(value).collect::<Vec<_>>(),
    ))
}

fn int_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> u32,
{
    Arc::new(Int32Array::from(
        manifests
            .iter()
            .map(|manifest| i32::try_from(value(manifest)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

fn float_manifest_column<F>(manifests: &[PdfPageShardManifest], value: F) -> ArrayRef
where
    F: Fn(&PdfPageShardManifest) -> f64,
{
    Arc::new(Float64Array::from(
        manifests.iter().map(value).collect::<Vec<_>>(),
    ))
}
pub(super) fn write_shard_artifact_batches(
    output_dir: &Path,
    manifests: &[PdfPageShardManifest],
    manifest_batch: RecordBatch,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let ocr_inputs = build_ocr_shard_inputs(manifests, &PdfOcrWorkerProfile::docling_compatible());
    let ocr_input_batch = build_ocr_shard_input_batch(&ocr_inputs)?;
    let pending_batch = build_ocr_pending_resource_batch(manifests)?;
    let manifest_arrow_path = output_dir.join(OCR_SHARD_MANIFEST_ARROW_NAME);
    let ocr_input_arrow_path = output_dir.join(OCR_SHARD_INPUT_ARROW_NAME);
    let pending_resource_arrow_path = output_dir.join(OCR_PENDING_RESOURCE_ARROW_NAME);
    write_arrow_file(manifest_arrow_path.as_path(), &[manifest_batch])?;
    write_arrow_file(ocr_input_arrow_path.as_path(), &[ocr_input_batch])?;
    write_arrow_file(pending_resource_arrow_path.as_path(), &[pending_batch])?;
    Ok((
        manifest_arrow_path,
        ocr_input_arrow_path,
        pending_resource_arrow_path,
    ))
}

fn shard_manifest_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, false),
        Field::new("sourceContentHash", DataType::Utf8, false),
        Field::new("pageIndex", DataType::Int32, false),
        Field::new("renderProfile", DataType::Utf8, false),
        Field::new("imagePath", DataType::Utf8, false),
        Field::new("imageMimeType", DataType::Utf8, false),
        Field::new("rasterSha256", DataType::Utf8, false),
        Field::new("rasterWidthPx", DataType::Int32, false),
        Field::new("rasterHeightPx", DataType::Int32, false),
        Field::new("renderDpi", DataType::Int32, false),
        Field::new("rotationDegrees", DataType::Int32, false),
        Field::new("mediaLeft", DataType::Float64, false),
        Field::new("mediaBottom", DataType::Float64, false),
        Field::new("mediaRight", DataType::Float64, false),
        Field::new("mediaTop", DataType::Float64, false),
        Field::new("cropLeft", DataType::Float64, false),
        Field::new("cropBottom", DataType::Float64, false),
        Field::new("cropRight", DataType::Float64, false),
        Field::new("cropTop", DataType::Float64, false),
        Field::new("pointToPixelScaleX", DataType::Float64, false),
        Field::new("pointToPixelScaleY", DataType::Float64, false),
        Field::new("elementId", DataType::Utf8, false),
        Field::new("shardType", DataType::Utf8, false),
        Field::new("regionIndex", DataType::Int32, false),
        Field::new("parentShardElementId", DataType::Utf8, false),
        Field::new("readingOrderKey", DataType::Utf8, false),
        Field::new("sourcePagePixelLeft", DataType::Int32, false),
        Field::new("sourcePagePixelTop", DataType::Int32, false),
        Field::new("sourcePagePixelRight", DataType::Int32, false),
        Field::new("sourcePagePixelBottom", DataType::Int32, false),
    ]))
}

fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

fn write_arrow_file(path: &Path, batches: &[RecordBatch]) -> Result<(), String> {
    if batches.is_empty() {
        return Err(format!(
            "cannot write empty Arrow IPC file `{}`",
            path.display()
        ));
    }
    let file = File::create(path)
        .map_err(|error| format!("create Arrow IPC file `{}`: {error}", path.display()))?;
    let mut writer = FileWriter::try_new(file, batches[0].schema().as_ref())
        .map_err(|error| format!("create Arrow IPC writer `{}`: {error}", path.display()))?;
    for batch in batches {
        writer
            .write(batch)
            .map_err(|error| format!("write Arrow IPC batch `{}`: {error}", path.display()))?;
    }
    writer
        .finish()
        .map_err(|error| format!("finish Arrow IPC file `{}`: {error}", path.display()))
}
