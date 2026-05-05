use std::path::Path;

use super::{
    RenderPageSelection, resolve_page_selection, sample_region_manifest,
    write_shard_artifact_batches,
};
use crate::pdf::render::{PdfPageRenderSelection, build_shard_manifest_batch};

#[test]
fn document_extract_pdf_render_writes_region_arrow_artifacts() -> Result<(), String> {
    let manifest = sample_region_manifest()?;
    let manifest_batch = build_shard_manifest_batch(std::slice::from_ref(&manifest))?;
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;

    let (manifest_path, input_path, pending_path) = write_shard_artifact_batches(
        temp_dir.path(),
        std::slice::from_ref(&manifest),
        manifest_batch,
    )?;

    assert!(manifest_path.is_file());
    assert!(input_path.is_file());
    assert!(pending_path.is_file());
    Ok(())
}

#[test]
fn document_extract_pdf_render_shard_fallback_defaults_to_all_pages_without_detector()
-> Result<(), String> {
    let selection = resolve_page_selection(
        Path::new("fixture.pdf"),
        PdfPageRenderSelection::ShardFallbackPages,
    )?;

    assert!(matches!(selection, RenderPageSelection::All));
    Ok(())
}
