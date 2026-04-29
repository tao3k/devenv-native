use std::path::PathBuf;

use xiuxian_wendao::gateway::studio::document_extract_pdf_audit::{
    PdfInspectorTextFastPathConfig, audit_pdf_paths, extract_text_pdf_fast_path_artifacts,
    read_audit_paths_from_json, write_audit_reports, write_text_fast_path_reports,
};

#[test]
#[ignore = "requires real PDF fixtures supplied by the document extraction benchmark harness"]
fn pdf_inspector_detect_audit_reports_pdf_routing_candidates() -> Result<(), String> {
    let inputs_json = std::env::var("WENDAO_PDF_INSPECTOR_AUDIT_INPUTS_JSON")
        .map_err(|_| "WENDAO_PDF_INSPECTOR_AUDIT_INPUTS_JSON is required".to_string())?;
    let report_dir = std::env::var("WENDAO_PDF_INSPECTOR_AUDIT_REPORT_DIR").map_or_else(
        |_| {
            PathBuf::from(
                ".run/reports/xiuxian-wendao/document-extract-perf/pdf-inspector-detect-audit",
            )
        },
        PathBuf::from,
    );

    let paths = read_audit_paths_from_json(inputs_json.as_str())?;
    let records = audit_pdf_paths(&paths);
    write_audit_reports(report_dir.as_path(), &records)?;
    let text_fast_path_records = extract_text_pdf_fast_path_artifacts(
        &paths,
        report_dir.join("text-fast-path-artifacts").as_path(),
        &PdfInspectorTextFastPathConfig::enabled(),
    );
    write_text_fast_path_reports(report_dir.as_path(), &text_fast_path_records)?;
    if records.is_empty() {
        return Err("PDF inspector audit produced no records".to_string());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "audit": records,
            "textFastPath": text_fast_path_records,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}
