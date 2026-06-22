//! Hosted VLM/OCR region render materialization.

mod materialization;

#[cfg(all(test, feature = "document-extract-pdf-render"))]
mod tests;

pub(super) use materialization::{
    materialize_ocr2_recovery_page_images, materialize_ocr2_recovery_region_images,
};

#[cfg(feature = "document-extract-pdf-render")]
pub(super) use materialization::{
    cached_ocr2_region_render_report, ocr2_recovery_region_requests_for_inputs,
    ocr2_region_render_cache_dir_with_source_hash, sha256_file_hex,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use materialization::{
    has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs,
};

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use materialization::OCR2_REGION_SCAFFOLD_FILE_NAME;

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(super) use materialization::{
    ocr2_region_render_cache_key, ocr2_region_render_cache_key_with_source_hash,
    ocr2_region_scaffold_payload, sha256_hex,
};
