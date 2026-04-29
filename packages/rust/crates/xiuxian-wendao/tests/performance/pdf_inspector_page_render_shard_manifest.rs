use std::path::PathBuf;

use xiuxian_wendao::gateway::studio::document_extract_pdf_render::{
    PdfPageRenderProfile, read_render_paths_from_json, render_pdf_page_shards,
    write_page_render_shard_reports,
};

#[test]
#[ignore = "requires real PDF fixtures and optionally a PDFium runtime library"]
fn pdf_inspector_page_render_shard_manifest_reports_pdf_shards() -> Result<(), String> {
    let inputs_json = std::env::var("WENDAO_PDF_RENDER_SHARD_INPUTS_JSON")
        .or_else(|_| std::env::var("WENDAO_PDF_INSPECTOR_AUDIT_INPUTS_JSON"))
        .map_err(|_| {
            "WENDAO_PDF_RENDER_SHARD_INPUTS_JSON or WENDAO_PDF_INSPECTOR_AUDIT_INPUTS_JSON is required"
                .to_string()
        })?;
    let report_dir = std::env::var("WENDAO_PDF_RENDER_SHARD_REPORT_DIR").map_or_else(
        |_| {
            PathBuf::from(
                ".run/reports/xiuxian-wendao/document-extract-perf/pdf-render-shard-manifest",
            )
        },
        PathBuf::from,
    );
    let artifact_dir = report_dir.join("artifacts");
    let profile = PdfPageRenderProfile::ocr_default();

    let paths = read_render_paths_from_json(inputs_json.as_str())?;
    let records = paths
        .iter()
        .map(|path| {
            let output_dir = artifact_dir.join(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("pdf"),
            );
            render_pdf_page_shards(path, output_dir.as_path(), &profile)
        })
        .collect::<Result<Vec<_>, _>>()?;
    write_page_render_shard_reports(report_dir.as_path(), &records)?;
    if records.is_empty() {
        return Err("PDF render shard manifest audit produced no records".to_string());
    }
    if std::env::var("WENDAO_PDF_RENDER_REQUIRE_PDFIUM").as_deref() == Ok("1")
        && records.iter().all(|record| record.status != "rendered")
    {
        return Err("PDFium was required but no PDF render shards were produced".to_string());
    }
    for record in records.iter().filter(|record| record.status == "rendered") {
        for artifact_path in [
            record.manifest_arrow_path.as_deref(),
            record.ocr_input_arrow_path.as_deref(),
            record.pending_resource_arrow_path.as_deref(),
        ] {
            let artifact_path = artifact_path
                .ok_or_else(|| "rendered record is missing Arrow artifact".to_string())?;
            if !PathBuf::from(artifact_path).is_file() {
                return Err(format!(
                    "rendered Arrow artifact does not exist: {artifact_path}"
                ));
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "records": records,
        }))
        .map_err(|error| error.to_string())?
    );
    Ok(())
}
