//! Hosted VLM/OCR region render materialization.

mod materialization;

#[cfg(all(test, feature = "document-extract-pdf-render"))]
mod tests;

#[cfg(any(feature = "document-extract-pdf-render", test))]
pub(super) use materialization::{
    cached_ocr2_region_render_report, materialize_ocr2_recovery_page_images,
    materialize_ocr2_recovery_region_images, ocr2_recovery_region_requests_for_inputs,
    ocr2_region_render_cache_dir_with_source_hash, sha256_file_hex,
    write_ocr2_region_scaffold_sidecar_with_lookup,
};

#[cfg(all(test, feature = "document-extract-pdf-render"))]
pub(crate) use materialization::{
    has_ocr2_recovery_page_candidates, merge_ocr2_recovery_page_inputs,
};

#[cfg(test)]
pub(crate) use materialization::OCR2_REGION_SCAFFOLD_FILE_NAME;

#[cfg(test)]
pub(super) use materialization::{
    ocr2_region_render_cache_key, ocr2_region_render_cache_key_with_source_hash,
    ocr2_region_scaffold_payload, sha256_hex,
};
