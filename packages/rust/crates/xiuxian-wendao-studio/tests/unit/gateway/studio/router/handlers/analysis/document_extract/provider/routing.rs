use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "document-extract-pdf-source-range")]
use super::{
    DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV, DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV,
    PdfPageRenderSelection, PdfRenderRoutingDecision, PdfRenderStatus,
    hybrid_page_ocr_input_arrow_path, hybrid_page_ocr_region_requests_for_source_with_lookup,
    hybrid_page_ocr_render_selection_with_lookup, sample_hybrid_page_ocr_report, sample_ocr_input,
    sample_ocr_result, validate_hybrid_page_coverage, validate_hybrid_shard_coverage,
    validate_ocr_results_match_inputs, validate_successful_ocr_results,
};

#[test]
fn document_extract_scope_forbids_relative_ancestor_visibility() -> Result<(), String> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/studio/router/handlers/analysis/document_extract");
    let mut files = Vec::new();
    collect_rust_source_files(source_root.as_path(), &mut files)
        .map_err(|error| error.to_string())?;
    let mut violations = Vec::new();
    for path in &files {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("read `{}`: {error}", path.display()))?;
        violations.extend(
            content
                .lines()
                .enumerate()
                .filter_map(|(line_index, line)| {
                    let declaration = line.trim();
                    declaration
                        .contains("pub(in super::")
                        .then(|| format!("{}:{} :: {declaration}", path.display(), line_index + 1))
                }),
        );
    }

    assert!(
        violations.is_empty(),
        "document_extract must not use relative ancestor visibility:\n{}",
        violations.join("\n")
    );
    Ok(())
}

fn collect_rust_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(path.as_path(), files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_defaults_to_shard_fallback() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|_| None);

    assert_eq!(selection, PdfPageRenderSelection::ShardFallbackPages);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_accepts_all_pages_override() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "all-pages".to_string())
    });

    assert_eq!(selection, PdfPageRenderSelection::AllPages);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_render_selection_accepts_region_shards_override() {
    let selection = hybrid_page_ocr_render_selection_with_lookup(&|key| {
        (key == DOCUMENT_EXTRACT_PDF_RENDER_SELECTION_ENV).then(|| "region-shards".to_string())
    });

    assert_eq!(selection, PdfPageRenderSelection::RegionShards);
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_requests_match_selected_source() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("source.pdf");
    fs::write(source.as_path(), b"%PDF").map_err(|error| error.to_string())?;
    let source_text = source.to_string_lossy();
    let regions_json = format!(
        r#"[{{"source":"{source_text}","regions":[{{"pageIndex":0,"regionIndex":3,"regionBox":{{"left":72.0,"bottom":72.0,"right":144.0,"top":144.0}},"readingOrderKey":"000000.000003"}}]}}]"#
    );

    let regions =
        hybrid_page_ocr_region_requests_for_source_with_lookup(source.as_path(), &|key| {
            (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.clone())
        })?;

    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].page_index, 0);
    assert_eq!(regions[0].region_index, 3);
    assert_eq!(
        regions[0].reading_order_key.as_deref(),
        Some("000000.000003")
    );
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_requests_reject_missing_source() {
    let regions_json = r#"[{"source":"/tmp/other.pdf","regions":[{"pageIndex":0,"regionIndex":0,"regionBox":{"left":0.0,"bottom":0.0,"right":10.0,"top":10.0}}]}]"#;

    let Err(error) = hybrid_page_ocr_region_requests_for_source_with_lookup(
        Path::new("/tmp/source.pdf"),
        &|key| (key == DOCUMENT_EXTRACT_PDF_RENDER_REGIONS_ENV).then(|| regions_json.into()),
    ) else {
        panic!("expected missing source to fail");
    };

    assert!(error.contains("no hybrid PDF region fixture matched"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_accepts_complete_render() -> Result<(), String> {
    let report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Rendered,
        PdfRenderRoutingDecision::HybridPageOcrCandidate,
        2,
        2,
        Some("/tmp/out/_ocr_input.arrow"),
    );

    let path = hybrid_page_ocr_input_arrow_path(&report)?;

    assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_accepts_partial_page_render() -> Result<(), String> {
    let report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Rendered,
        PdfRenderRoutingDecision::HybridPageOcrCandidate,
        3,
        1,
        Some("/tmp/out/_ocr_input.arrow"),
    );

    let path = hybrid_page_ocr_input_arrow_path(&report)?;

    assert_eq!(path, PathBuf::from("/tmp/out/_ocr_input.arrow"));
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_input_arrow_path_rejects_fallback_report() {
    let report = sample_hybrid_page_ocr_report(
        PdfRenderStatus::Fallback,
        PdfRenderRoutingDecision::FullDoclingFallback,
        1,
        0,
        None,
    );

    let Err(error) = hybrid_page_ocr_input_arrow_path(&report) else {
        panic!("fallback report should not become OCR input");
    };

    assert!(error.contains("not eligible for hybrid OCR"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_validation_rejects_skipped_ocr_rows() {
    let Err(error) = validate_successful_ocr_results(&[sample_ocr_result(1, false)], 3, 1) else {
        panic!("expected non-success OCR status to fail");
    };

    assert!(error.contains("non-success status"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_accepts_text_and_ocr_pages() -> Result<(), String> {
    validate_hybrid_page_coverage(3, &[0, 2], &[sample_ocr_result(1, true)])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_accepts_text_only_pages() -> Result<(), String> {
    validate_hybrid_page_coverage(3, &[0, 1, 2], &[])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_rejects_missing_pages() {
    let Err(error) = validate_hybrid_page_coverage(3, &[0], &[sample_ocr_result(1, true)]) else {
        panic!("expected missing page coverage to fail");
    };

    assert!(error.contains("missing page coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_coverage_rejects_duplicate_pages() {
    let Err(error) = validate_hybrid_page_coverage(3, &[0, 1], &[sample_ocr_result(1, true)])
    else {
        panic!("expected duplicate page coverage to fail");
    };

    assert!(error.contains("duplicate page coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_keeps_native_text_page() -> Result<(), String> {
    let input = sample_ocr_input(1, "region");
    let result = sample_ocr_result(1, true);

    validate_hybrid_shard_coverage(3, &[0, 1, 2], &[input], &[result])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_region_coverage_requires_native_text_page() {
    let input = sample_ocr_input(1, "region");
    let result = sample_ocr_result(1, true);

    let Err(error) = validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result]) else {
        panic!("expected region without native text page to fail");
    };

    assert!(error.contains("has no native text coverage"));
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_page_shard_coverage_still_replaces_full_page() -> Result<(), String> {
    let input = sample_ocr_input(1, "page");
    let result = sample_ocr_result(1, true);

    validate_hybrid_shard_coverage(3, &[0, 2], &[input], &[result])
}

#[cfg(feature = "document-extract-pdf-source-range")]
#[test]
fn hybrid_page_ocr_validation_rejects_unknown_shard_result() {
    let input = sample_ocr_input(1, "region");
    let mut result = sample_ocr_result(1, true);
    result.shard_element_id = "unknown-shard".to_string();

    let Err(error) = validate_ocr_results_match_inputs(&[input], &[result]) else {
        panic!("expected unknown OCR result shard to fail");
    };

    assert!(error.contains("unknown shard id"));
}
