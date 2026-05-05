use std::{collections::BTreeMap, path::PathBuf};

use xiuxian_wendao_studio::studio::document_extract_pdf_render::{
    PdfPageRegionRenderRequest, PdfPageRenderProfile, PdfPageRenderSelection,
    read_render_paths_from_json, render_pdf_page_shards_with_selection, render_pdf_region_shards,
    write_page_render_shard_reports,
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegionInput {
    source: PathBuf,
    regions: Vec<PdfPageRegionRenderRequest>,
}

#[test]
#[ignore = "requires real PDF fixtures and optionally a PDFium runtime library"]
fn pdf_render_page_render_shard_manifest_reports_pdf_shards() -> Result<(), String> {
    let inputs_json = std::env::var("WENDAO_PDF_RENDER_SHARD_INPUTS_JSON")
        .map_err(|_| "WENDAO_PDF_RENDER_SHARD_INPUTS_JSON is required".to_string())?;
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
    let selection = read_render_selection_from_env()?;

    let paths = read_render_paths_from_json(inputs_json.as_str())?;
    let regions_by_path = read_region_requests_from_env(selection)?;
    ensure_region_inputs_match_paths(selection, &paths, &regions_by_path)?;
    let records = paths
        .iter()
        .map(|path| {
            let output_dir = artifact_dir.join(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("pdf"),
            );
            match selection {
                PdfPageRenderSelection::RegionShards => {
                    let regions = regions_by_path.get(path).ok_or_else(|| {
                        format!(
                            "missing PDF render region requests for `{}`",
                            path.display()
                        )
                    })?;
                    render_pdf_region_shards(path, output_dir.as_path(), &profile, regions)
                }
                PdfPageRenderSelection::AllPages | PdfPageRenderSelection::ShardFallbackPages => {
                    render_pdf_page_shards_with_selection(
                        path,
                        output_dir.as_path(),
                        &profile,
                        selection,
                    )
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    write_page_render_shard_reports(report_dir.as_path(), &records)?;
    if records.is_empty() {
        return Err("PDF render shard manifest audit produced no records".to_string());
    }
    if matches!(
        selection,
        PdfPageRenderSelection::AllPages | PdfPageRenderSelection::RegionShards
    ) && std::env::var("WENDAO_PDF_RENDER_REQUIRE_PDFIUM").as_deref() == Ok("1")
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

fn read_render_selection_from_env() -> Result<PdfPageRenderSelection, String> {
    match std::env::var("WENDAO_PDF_RENDER_SELECTION")
        .unwrap_or_else(|_| PdfPageRenderSelection::AllPages.as_str().to_string())
        .as_str()
    {
        "all_pages" => Ok(PdfPageRenderSelection::AllPages),
        "shard_fallback_pages" => Ok(PdfPageRenderSelection::ShardFallbackPages),
        "region_shards" => Ok(PdfPageRenderSelection::RegionShards),
        value => Err(format!("unsupported PDF render selection: {value}")),
    }
}

fn read_region_requests_from_env(
    selection: PdfPageRenderSelection,
) -> Result<BTreeMap<PathBuf, Vec<PdfPageRegionRenderRequest>>, String> {
    if selection != PdfPageRenderSelection::RegionShards {
        return Ok(BTreeMap::new());
    }
    let regions_json = std::env::var("WENDAO_PDF_RENDER_REGIONS_JSON").map_err(|_| {
        "WENDAO_PDF_RENDER_REGIONS_JSON is required for region_shards selection".to_string()
    })?;
    let mut regions_by_path = BTreeMap::new();
    for input in serde_json::from_str::<Vec<RegionInput>>(regions_json.as_str())
        .map_err(|error| format!("parse PDF render region JSON: {error}"))?
    {
        if input.regions.is_empty() {
            return Err(format!(
                "PDF render region fixture has no regions: {}",
                input.source.display()
            ));
        }
        if regions_by_path
            .insert(input.source.clone(), input.regions)
            .is_some()
        {
            return Err(format!(
                "duplicate PDF render region fixture: {}",
                input.source.display()
            ));
        }
    }
    Ok(regions_by_path)
}

fn ensure_region_inputs_match_paths(
    selection: PdfPageRenderSelection,
    paths: &[PathBuf],
    regions_by_path: &BTreeMap<PathBuf, Vec<PdfPageRegionRenderRequest>>,
) -> Result<(), String> {
    if selection != PdfPageRenderSelection::RegionShards {
        return Ok(());
    }
    for path in paths {
        if !regions_by_path.contains_key(path) {
            return Err(format!(
                "missing PDF render region fixture for input: {}",
                path.display()
            ));
        }
    }
    for path in regions_by_path.keys() {
        if !paths.contains(path) {
            return Err(format!(
                "PDF render region fixture does not match selected input: {}",
                path.display()
            ));
        }
    }
    Ok(())
}
