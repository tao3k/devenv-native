use super::{
    assert_close, float64_column, int32_column, sample_manifest, sample_region_manifest,
    string_column,
};
use crate::pdf::render::batches::shard_manifests_from_batch;
use crate::pdf::render::{build_ocr_pending_resource_batch, build_shard_manifest_batch};

#[test]
fn document_extract_pdf_render_builds_typed_manifest_arrow_batch() -> Result<(), String> {
    let batch = build_shard_manifest_batch(&[sample_manifest(180)])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.schema().field(0).name(), "sourcePath");
    assert_eq!(batch.schema().field(1).name(), "sourceContentHash");
    assert_eq!(batch.schema().field(11).name(), "mediaLeft");
    assert_eq!(int32_column(&batch, "rotationDegrees")?.value(0), 180);
    assert_close(float64_column(&batch, "cropLeft")?.value(0), 18.0);
    assert_eq!(string_column(&batch, "shardType")?.value(0), "page");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 0);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000002.000000"
    );
    assert_eq!(int32_column(&batch, "sourcePagePixelLeft")?.value(0), 0);
    Ok(())
}

#[test]
fn document_extract_pdf_render_builds_region_manifest_arrow_batch() -> Result<(), String> {
    let batch = build_shard_manifest_batch(&[sample_region_manifest()?])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(string_column(&batch, "shardType")?.value(0), "region");
    assert_eq!(int32_column(&batch, "regionIndex")?.value(0), 7);
    assert_eq!(
        string_column(&batch, "readingOrderKey")?.value(0),
        "000002.000007"
    );
    assert_eq!(
        int32_column(&batch, "sourcePagePixelBottom")?.value(0),
        2325
    );
    Ok(())
}

#[test]
fn document_extract_pdf_render_decodes_region_manifest_arrow_batch() -> Result<(), String> {
    let manifest = sample_region_manifest()?;
    let batch = build_shard_manifest_batch(std::slice::from_ref(&manifest))?;
    let restored = shard_manifests_from_batch(&batch)?;

    assert_eq!(restored, vec![manifest]);
    Ok(())
}

#[test]
fn document_extract_pdf_render_builds_ocr_pending_resource_rows() -> Result<(), String> {
    let batch = build_ocr_pending_resource_batch(&[sample_manifest(0)])?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(
        string_column(&batch, "resourceType")?.value(0),
        "ocr_pending"
    );
    assert_eq!(string_column(&batch, "status")?.value(0), "pending");
    assert!(
        string_column(&batch, "content")?
            .value(0)
            .contains("_ocr_shards.arrow")
    );
    assert!(
        string_column(&batch, "content")?
            .value(0)
            .contains("shard_type=page")
    );
    Ok(())
}
