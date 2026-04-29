use super::*;
use arrow::array::{Int32Array, StringArray};
use arrow::record_batch::RecordBatch;
use pdf_inspector::{PageMarkdown, PagesExtractionResult};

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

fn int32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("`{name}` column is not Int32"))
}

fn signals(pdf_type: PdfInspectorPdfType) -> PdfInspectorRoutingSignals {
    PdfInspectorRoutingSignals {
        pdf_type,
        page_count: 4,
        confidence: 0.95,
        pages_needing_ocr: Vec::new(),
        is_complex: false,
        has_encoding_issues: false,
    }
}

fn minimal_text_pdf_add_object(pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, id: usize, body: &str) {
    offsets.push(pdf.len());
    pdf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
    pdf.extend_from_slice(body.as_bytes());
    pdf.extend_from_slice(b"\nendobj\n");
}

fn minimal_text_pdf() -> Vec<u8> {
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0_usize];

    minimal_text_pdf_add_object(
        &mut pdf,
        &mut offsets,
        1,
        "<< /Type /Catalog /Pages 2 0 R >>",
    );
    minimal_text_pdf_add_object(
        &mut pdf,
        &mut offsets,
        2,
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    );
    minimal_text_pdf_add_object(
        &mut pdf,
        &mut offsets,
        3,
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
    );
    let content = "BT /F1 12 Tf 100 700 Td (Hello World) Tj 0 -14 Td (Second Line) Tj 0 -14 Td (Third Line) Tj ET";
    minimal_text_pdf_add_object(
        &mut pdf,
        &mut offsets,
        4,
        &format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        ),
    );
    minimal_text_pdf_add_object(
        &mut pdf,
        &mut offsets,
        5,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    );

    let xref_start = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", offsets.len()).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            offsets.len(),
            xref_start
        )
        .as_bytes(),
    );

    pdf
}

#[test]
fn document_extract_pdf_audit_routes_simple_text_pdf_to_fast_candidate() {
    assert_eq!(
        routing_decision(&signals(PdfInspectorPdfType::TextBased)),
        PdfInspectorRoutingDecision::FastRustCandidate
    );
}

#[test]
fn document_extract_pdf_audit_routes_mixed_pdf_to_hybrid_candidate() {
    let mut input = signals(PdfInspectorPdfType::Mixed);
    input.pages_needing_ocr = vec![2];

    assert_eq!(
        routing_decision(&input),
        PdfInspectorRoutingDecision::HybridPageOcrCandidate
    );
}

#[test]
fn document_extract_pdf_audit_routes_scanned_and_image_pdf_to_hybrid_candidate() {
    for pdf_type in [
        PdfInspectorPdfType::Scanned,
        PdfInspectorPdfType::ImageBased,
    ] {
        let mut input = signals(pdf_type);
        input.pages_needing_ocr = vec![1, 2, 3, 4];

        assert_eq!(
            routing_decision(&input),
            PdfInspectorRoutingDecision::HybridPageOcrCandidate
        );
    }
}

#[test]
fn document_extract_pdf_audit_routes_low_confidence_to_docling_fallback() {
    let mut input = signals(PdfInspectorPdfType::TextBased);
    input.confidence = 0.5;

    assert_eq!(
        routing_decision(&input),
        PdfInspectorRoutingDecision::FullDoclingFallback
    );
    let assessment = routing_assessment(&input);
    assert_eq!(
        assessment.gate_failures,
        vec![PdfInspectorRoutingGateFailure::LowConfidence]
    );
    assert!(assessment.fast_path_score < 0.90);
}

#[test]
fn document_extract_pdf_audit_routes_encoding_issues_to_docling_fallback() {
    let mut input = signals(PdfInspectorPdfType::TextBased);
    input.has_encoding_issues = true;

    assert_eq!(
        routing_decision(&input),
        PdfInspectorRoutingDecision::FullDoclingFallback
    );
}

#[test]
fn document_extract_pdf_audit_routes_complex_text_pdf_to_hybrid_shard_candidate() {
    let mut input = signals(PdfInspectorPdfType::TextBased);
    input.is_complex = true;

    assert_eq!(
        routing_decision(&input),
        PdfInspectorRoutingDecision::HybridPageOcrCandidate
    );
    let assessment = routing_assessment(&input);
    assert_eq!(
        assessment.gate_failures,
        vec![PdfInspectorRoutingGateFailure::ComplexLayout]
    );
    assert!(assessment.fast_path_score < 0.90);
}

#[test]
fn document_extract_pdf_audit_routes_non_text_without_page_hints_to_hybrid_shard_candidate() {
    let input = signals(PdfInspectorPdfType::ImageBased);

    let assessment = routing_assessment(&input);

    assert_eq!(
        assessment.decision,
        PdfInspectorRoutingDecision::HybridPageOcrCandidate
    );
    assert_eq!(
        assessment.gate_failures,
        vec![PdfInspectorRoutingGateFailure::NonTextPdf]
    );
}

#[test]
fn document_extract_pdf_audit_explains_high_confidence_scanned_hybrid_candidate() {
    let mut input = signals(PdfInspectorPdfType::Scanned);
    input.pages_needing_ocr = vec![1, 2, 3, 4];

    let assessment = routing_assessment(&input);

    assert_eq!(
        assessment.decision,
        PdfInspectorRoutingDecision::HybridPageOcrCandidate
    );
    assert_eq!(
        assessment.gate_failures,
        vec![
            PdfInspectorRoutingGateFailure::NonTextPdf,
            PdfInspectorRoutingGateFailure::PagesNeedOcr,
        ]
    );
    assert!(assessment.fast_path_score < 0.90);
}

#[test]
fn document_extract_pdf_audit_routes_low_confidence_ocr_pdf_to_hybrid_shard_candidate() {
    let mut input = signals(PdfInspectorPdfType::Mixed);
    input.confidence = 0.5;
    input.pages_needing_ocr = vec![2];

    let assessment = routing_assessment(&input);

    assert_eq!(
        assessment.decision,
        PdfInspectorRoutingDecision::HybridPageOcrCandidate
    );
    assert_eq!(
        assessment.gate_failures,
        vec![
            PdfInspectorRoutingGateFailure::LowConfidence,
            PdfInspectorRoutingGateFailure::NonTextPdf,
            PdfInspectorRoutingGateFailure::PagesNeedOcr,
        ]
    );
}

#[test]
fn document_extract_pdf_audit_marks_non_pdf_as_unsupported() {
    let records = audit_pdf_path(Path::new("sample.docx"));

    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].routing_decision,
        PdfInspectorRoutingDecision::UnsupportedNonPdf.as_str()
    );
    assert_eq!(records[0].gate_failures, vec!["unsupported_non_pdf"]);
}

#[test]
fn document_extract_pdf_audit_marks_invalid_pdf_as_preflight_failed() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("broken.pdf");
    fs::write(source.as_path(), b"not a pdf").map_err(|error| error.to_string())?;

    let records = audit_pdf_path(source.as_path());

    assert_eq!(records.len(), 2);
    assert!(
        records.iter().all(|record| record.routing_decision
            == PdfInspectorRoutingDecision::PreflightFailed.as_str())
    );
    assert!(
        records
            .iter()
            .all(|record| record.gate_failures == vec!["preflight_failed"])
    );
    Ok(())
}

#[test]
fn document_extract_pdf_text_fast_path_can_be_disabled_by_config() {
    let record = extract_text_pdf_fast_path_artifact(
        Path::new("sample.pdf"),
        Path::new("sample.pdf.extracted"),
        &PdfInspectorTextFastPathConfig::disabled(),
    );

    assert_eq!(record.status, "disabled");
    assert_eq!(record.converter_profile, "pdf-inspector-text-fast-path-v1");
    assert_eq!(record.arrow_rows, 0);
}

#[test]
fn document_extract_pdf_text_fast_path_writes_stable_arrow_resource_row() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("sample.pdf");
    let output_dir = temp.path().join("sample.pdf.extracted");
    fs::write(source.as_path(), b"%PDF-1.4\n").map_err(|error| error.to_string())?;

    let (markdown_path, arrow_cache_path, batch) =
        write_text_fast_path_artifact(source.as_path(), output_dir.as_path(), "# Sample\n")?;

    assert_eq!(markdown_path, output_dir.join("sample.md"));
    assert_eq!(
        arrow_cache_path,
        output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
    );
    assert!(
        output_dir
            .join(DOCUMENT_EXTRACT_COMPLETE_MARKER_NAME)
            .exists()
    );
    assert_eq!(batch.num_rows(), 1);
    let schema = batch.schema();
    let field_names = schema
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        field_names,
        vec![
            "sourcePath",
            "resourceType",
            "resourcePath",
            "pageIndex",
            "caption",
            "content",
            "mimeType",
            "status",
            "elementId",
        ]
    );
    Ok(())
}

#[test]
fn document_extract_pdf_text_page_resources_skip_ocr_pages() -> Result<(), String> {
    let source = Path::new("/tmp/mixed.pdf");
    let pages = PagesExtractionResult {
        pages: vec![
            PageMarkdown {
                page: 0,
                markdown: "First text page".to_string(),
                needs_ocr: false,
            },
            PageMarkdown {
                page: 1,
                markdown: String::new(),
                needs_ocr: true,
            },
            PageMarkdown {
                page: 2,
                markdown: "Third text page".to_string(),
                needs_ocr: false,
            },
        ],
        pages_with_tables: Vec::new(),
        pages_with_columns: Vec::new(),
        pages_needing_ocr: vec![2],
        is_complex: false,
    };

    let resource_batch = build_text_page_resource_batch(source, &pages, &[1])?;

    assert_eq!(resource_batch.page_indices, vec![0, 2]);
    assert_eq!(resource_batch.batch.num_rows(), 2);
    assert_eq!(
        string_column(&resource_batch.batch, "resourceType")?.value(0),
        "text_page"
    );
    assert_eq!(
        int32_column(&resource_batch.batch, "pageIndex")?.value(0),
        0
    );
    assert_eq!(
        int32_column(&resource_batch.batch, "pageIndex")?.value(1),
        2
    );
    assert_eq!(
        string_column(&resource_batch.batch, "content")?.value(1),
        "Third text page"
    );
    Ok(())
}

#[test]
fn document_extract_pdf_audit_selects_image_placeholder_pages_for_high_recall_ocr() {
    let pages = PagesExtractionResult {
        pages: vec![
            PageMarkdown {
                page: 0,
                markdown: "Native text".to_string(),
                needs_ocr: false,
            },
            PageMarkdown {
                page: 1,
                markdown: "Caption\n\n![Image: scan](image)".to_string(),
                needs_ocr: false,
            },
            PageMarkdown {
                page: 2,
                markdown: String::new(),
                needs_ocr: true,
            },
        ],
        pages_with_tables: Vec::new(),
        pages_with_columns: Vec::new(),
        pages_needing_ocr: Vec::new(),
        is_complex: true,
    };

    assert_eq!(
        high_recall_ocr_page_numbers_from_extraction(&pages),
        vec![2, 3]
    );
}

#[test]
fn document_extract_pdf_text_fast_path_extracts_simple_text_pdf() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let source = temp.path().join("simple.pdf");
    let output_dir = temp.path().join("simple.pdf.extracted");
    fs::write(source.as_path(), minimal_text_pdf()).map_err(|error| error.to_string())?;

    let record = extract_text_pdf_fast_path_artifact(
        source.as_path(),
        output_dir.as_path(),
        &PdfInspectorTextFastPathConfig::enabled(),
    );

    assert_eq!(record.status, "ok", "{record:?}");
    assert_eq!(record.routing_decision, "fast_rust_candidate");
    assert_eq!(record.arrow_rows, 1);
    assert!(record.markdown_bytes > 0);
    let markdown_path = record
        .artifact_path
        .as_deref()
        .ok_or_else(|| "missing markdown artifact path".to_string())?;
    let markdown =
        fs::read_to_string(markdown_path).map_err(|error| format!("read markdown: {error}"))?;
    assert!(markdown.contains("Hello World"));
    assert!(markdown.contains("Second Line"));
    assert!(record.arrow_cache_path.is_some());
    Ok(())
}
