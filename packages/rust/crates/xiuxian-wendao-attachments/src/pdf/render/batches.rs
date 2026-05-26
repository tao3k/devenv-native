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

/// Decode a typed shard manifest batch into manifest DTO rows.
///
/// # Errors
///
/// Returns an error if required columns are missing, typed incorrectly, or
/// contain unsupported shard-type values.
#[cfg(feature = "pdf-render")]
pub(super) fn shard_manifests_from_batch(
    batch: &RecordBatch,
) -> Result<Vec<PdfPageShardManifest>, String> {
    let source_path = string_column(batch, "sourcePath")?;
    let source_content_hash = string_column(batch, "sourceContentHash")?;
    let page_index = int_column(batch, "pageIndex")?;
    let render_profile = string_column(batch, "renderProfile")?;
    let image_path = string_column(batch, "imagePath")?;
    let image_mime_type = string_column(batch, "imageMimeType")?;
    let raster_sha256 = string_column(batch, "rasterSha256")?;
    let raster_width_px = int_column(batch, "rasterWidthPx")?;
    let raster_height_px = int_column(batch, "rasterHeightPx")?;
    let render_dpi = int_column(batch, "renderDpi")?;
    let rotation_degrees = int_column(batch, "rotationDegrees")?;
    let media_left = float_column(batch, "mediaLeft")?;
    let media_bottom = float_column(batch, "mediaBottom")?;
    let media_right = float_column(batch, "mediaRight")?;
    let media_top = float_column(batch, "mediaTop")?;
    let crop_left = float_column(batch, "cropLeft")?;
    let crop_bottom = float_column(batch, "cropBottom")?;
    let crop_right = float_column(batch, "cropRight")?;
    let crop_top = float_column(batch, "cropTop")?;
    let point_to_pixel_scale_x = float_column(batch, "pointToPixelScaleX")?;
    let point_to_pixel_scale_y = float_column(batch, "pointToPixelScaleY")?;
    let element_id = string_column(batch, "elementId")?;
    let shard_type = string_column(batch, "shardType")?;
    let region_index = int_column(batch, "regionIndex")?;
    let parent_shard_element_id = string_column(batch, "parentShardElementId")?;
    let reading_order_key = string_column(batch, "readingOrderKey")?;
    let source_page_pixel_left = int_column(batch, "sourcePagePixelLeft")?;
    let source_page_pixel_top = int_column(batch, "sourcePagePixelTop")?;
    let source_page_pixel_right = int_column(batch, "sourcePagePixelRight")?;
    let source_page_pixel_bottom = int_column(batch, "sourcePagePixelBottom")?;

    (0..batch.num_rows())
        .map(|row| {
            let shard_type = match shard_type.value(row) {
                "page" => PdfOcrShardType::Page,
                "region" => PdfOcrShardType::Region,
                value => return Err(format!("unsupported PDF render shard type `{value}`")),
            };
            Ok(PdfPageShardManifest {
                source_path: source_path.value(row).to_string(),
                source_content_hash: source_content_hash.value(row).to_string(),
                page_index: i32_to_u32(page_index.value(row), "pageIndex")?,
                shard_type,
                region_index: i32_to_u32(region_index.value(row), "regionIndex")?,
                parent_shard_element_id: parent_shard_element_id.value(row).to_string(),
                reading_order_key: reading_order_key.value(row).to_string(),
                render_profile: render_profile.value(row).to_string(),
                image_path: image_path.value(row).to_string(),
                image_mime_type: image_mime_type.value(row).to_string(),
                raster_sha256: raster_sha256.value(row).to_string(),
                geometry: super::types::PdfPageShardGeometry {
                    media_box: super::types::PdfPageBox::new(
                        media_left.value(row),
                        media_bottom.value(row),
                        media_right.value(row),
                        media_top.value(row),
                    ),
                    crop_box: super::types::PdfPageBox::new(
                        crop_left.value(row),
                        crop_bottom.value(row),
                        crop_right.value(row),
                        crop_top.value(row),
                    ),
                    rotation_degrees: i32_to_u16(rotation_degrees.value(row), "rotationDegrees")?,
                    render_dpi: i32_to_u32(render_dpi.value(row), "renderDpi")?,
                    raster_width_px: i32_to_u32(raster_width_px.value(row), "rasterWidthPx")?,
                    raster_height_px: i32_to_u32(raster_height_px.value(row), "rasterHeightPx")?,
                    point_to_pixel_scale_x: point_to_pixel_scale_x.value(row),
                    point_to_pixel_scale_y: point_to_pixel_scale_y.value(row),
                },
                source_page_pixel_box: super::types::PdfPagePixelBox::new(
                    i32_to_u32(source_page_pixel_left.value(row), "sourcePagePixelLeft")?,
                    i32_to_u32(source_page_pixel_top.value(row), "sourcePagePixelTop")?,
                    i32_to_u32(source_page_pixel_right.value(row), "sourcePagePixelRight")?,
                    i32_to_u32(source_page_pixel_bottom.value(row), "sourcePagePixelBottom")?,
                ),
                element_id: element_id.value(row).to_string(),
            })
        })
        .collect()
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

#[cfg(feature = "pdf-render")]
fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

#[cfg(feature = "pdf-render")]
fn int_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("`{name}` column is not Int32"))
}

#[cfg(feature = "pdf-render")]
fn float_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("`{name}` column is not Float64"))
}

#[cfg(feature = "pdf-render")]
fn i32_to_u32(value: i32, column: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("`{column}` value must be non-negative"))
}

#[cfg(feature = "pdf-render")]
fn i32_to_u16(value: i32, column: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("`{column}` value is outside u16 range"))
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
