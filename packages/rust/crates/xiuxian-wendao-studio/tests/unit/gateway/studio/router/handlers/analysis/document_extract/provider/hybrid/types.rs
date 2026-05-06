use super::{
    DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE_ENV, HybridPdfOcr2ScaffoldMode,
    hybrid_page_ocr2_scaffold_mode_with_lookup,
};

#[test]
fn scaffold_mode_accepts_region_table_json() {
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE_ENV)
                .then(|| "region_table_json".to_string())
        }),
        HybridPdfOcr2ScaffoldMode::RegionTableJson
    );
}

#[test]
fn scaffold_mode_defaults_unknown_values_to_disabled() {
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|_key| None),
        HybridPdfOcr2ScaffoldMode::Disabled
    );
    assert_eq!(
        hybrid_page_ocr2_scaffold_mode_with_lookup(&|key| {
            (key == DOCUMENT_EXTRACT_PDF_OCR2_SCAFFOLD_MODE_ENV).then(|| "unknown".to_string())
        }),
        HybridPdfOcr2ScaffoldMode::Disabled
    );
}
