use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::studio::router::handlers::analysis::document_extract::provider::legacy_office::{
    LegacyOfficeProjection, build_legacy_office_projection_batch,
    legacy_office_projection_from_batches,
};
use arrow::array::StringArray;
use xiuxian_wendao_attachments::legacy_office::{
    LegacyOfficeFormat, legacy_office_quality_metrics,
};

use super::{
    DOCUMENT_RESOURCE_ARROW_CACHE_NAME, is_legacy_office_source, read_arrow_file, write_arrow_file,
    write_legacy_office_document_extract_output,
};

#[test]
fn legacy_office_source_detection_routes_only_legacy_office() {
    assert!(is_legacy_office_source(Path::new("a.doc")));
    assert!(is_legacy_office_source(Path::new("a.XLS")));
    assert!(is_legacy_office_source(Path::new("a.ppt")));
    assert!(!is_legacy_office_source(Path::new("a.docx")));
}

#[test]
fn legacy_office_projection_restore_rejects_source_hash_mismatch() {
    let batch = match build_legacy_office_projection_batch(
        "expected",
        &sample_projection(LegacyOfficeFormat::Doc),
    ) {
        Ok(batch) => batch,
        Err(error) => panic!("build projection failed: {error}"),
    };

    let error =
        match legacy_office_projection_from_batches(&[batch], "actual", LegacyOfficeFormat::Doc) {
            Ok(_) => panic!("source hash mismatch should fail"),
            Err(error) => error,
        };

    assert!(error.contains("sourceSha256 mismatch"));
}

#[test]
fn legacy_office_projection_restore_rejects_format_mismatch() {
    let batch = match build_legacy_office_projection_batch(
        "same",
        &sample_projection(LegacyOfficeFormat::Doc),
    ) {
        Ok(batch) => batch,
        Err(error) => panic!("build projection failed: {error}"),
    };

    let error =
        match legacy_office_projection_from_batches(&[batch], "same", LegacyOfficeFormat::Xls) {
            Ok(_) => panic!("format mismatch should fail"),
            Err(error) => error,
        };

    assert!(error.contains("format mismatch"));
}

#[tokio::test]
async fn legacy_office_real_fixture_diagnostic_writes_resource_output_when_env_is_set()
-> Result<(), String> {
    let Some(path) = std::env::var_os("WENDAO_LEGACY_OFFICE_DIAGNOSTIC_PATH") else {
        return Ok(());
    };
    let result = run_legacy_office_diagnostic_source(Path::new(&path)).await?;
    eprintln!(
        "studio_legacy_office_diagnostic rows={} content_chars={} line_count={} tab_delimited_rows={} max_columns={} fenced_blocks={} cache_backend={} cache_status={}",
        result.rows,
        result.content_chars,
        result.line_count,
        result.tab_delimited_row_count,
        result.max_column_count,
        result.markdown_fenced_block_count,
        result.cache_backend,
        result.cache_status
    );
    Ok(())
}

#[tokio::test]
async fn legacy_office_real_fixture_matrix_diagnostic_when_env_is_set() -> Result<(), String> {
    let Some(root) = std::env::var_os("WENDAO_LEGACY_OFFICE_DIAGNOSTIC_ROOT") else {
        return Ok(());
    };
    let sources = collect_legacy_office_sources(Path::new(&root))?;
    if sources.is_empty() {
        return Err(
            "legacy Office diagnostic root did not contain .doc, .xls, or .ppt files".into(),
        );
    }

    let mut summary = LegacyOfficeDiagnosticSummary::default();
    for source in sources {
        let result = run_legacy_office_diagnostic_source(source.as_path()).await?;
        summary.record(&result);
    }

    assert_eq!(summary.failed, 0);
    assert_eq!(summary.total, summary.rows);
    assert!(summary.content_chars > 0);
    if std::env::var_os("WENDAO_ARTIFACT_CACHE_BACKEND").is_some() {
        assert_eq!(summary.disabled_cache_status, 0);
    }
    eprintln!(
        "studio_legacy_office_matrix_diagnostic total={} doc={} xls={} ppt={} rows={} content_chars={} line_count={} tab_delimited_rows={} max_columns={} fenced_blocks={} cache_hits={} cache_misses={} cache_disabled={}",
        summary.total,
        summary.format_counts.get("doc").copied().unwrap_or(0),
        summary.format_counts.get("xls").copied().unwrap_or(0),
        summary.format_counts.get("ppt").copied().unwrap_or(0),
        summary.rows,
        summary.content_chars,
        summary.line_count,
        summary.tab_delimited_row_count,
        summary.max_column_count,
        summary.markdown_fenced_block_count,
        summary.hit_cache_status,
        summary.miss_cache_status,
        summary.disabled_cache_status,
    );
    Ok(())
}

#[test]
fn legacy_office_projection_restore_rejects_empty_text() {
    let mut projection = sample_projection(LegacyOfficeFormat::Doc);
    projection.text.clear();
    let batch = match build_legacy_office_projection_batch("same", &projection) {
        Ok(batch) => batch,
        Err(error) => panic!("build projection failed: {error}"),
    };

    let error =
        match legacy_office_projection_from_batches(&[batch], "same", LegacyOfficeFormat::Doc) {
            Ok(_) => panic!("empty projection should fail"),
            Err(error) => error,
        };

    assert!(error.contains("empty text"));
}

#[derive(Debug)]
struct LegacyOfficeDiagnosticResult {
    format: String,
    rows: usize,
    content_chars: usize,
    line_count: u64,
    tab_delimited_row_count: u64,
    max_column_count: u64,
    markdown_fenced_block_count: u64,
    cache_backend: String,
    cache_status: String,
}

#[derive(Debug, Default)]
struct LegacyOfficeDiagnosticSummary {
    total: usize,
    failed: usize,
    rows: usize,
    content_chars: usize,
    line_count: u64,
    tab_delimited_row_count: u64,
    max_column_count: u64,
    markdown_fenced_block_count: u64,
    hit_cache_status: usize,
    miss_cache_status: usize,
    disabled_cache_status: usize,
    format_counts: BTreeMap<String, usize>,
}

impl LegacyOfficeDiagnosticSummary {
    fn record(&mut self, result: &LegacyOfficeDiagnosticResult) {
        self.total += 1;
        self.rows += result.rows;
        self.content_chars += result.content_chars;
        self.line_count += result.line_count;
        self.tab_delimited_row_count += result.tab_delimited_row_count;
        self.max_column_count = self.max_column_count.max(result.max_column_count);
        self.markdown_fenced_block_count += result.markdown_fenced_block_count;
        *self.format_counts.entry(result.format.clone()).or_default() += 1;
        match result.cache_status.as_str() {
            "Hit" => self.hit_cache_status += 1,
            "Miss" => self.miss_cache_status += 1,
            "Disabled" => self.disabled_cache_status += 1,
            _ => {}
        }
    }
}

async fn run_legacy_office_diagnostic_source(
    source: &Path,
) -> Result<LegacyOfficeDiagnosticResult, String> {
    let output = tempfile::tempdir().map_err(|error| error.to_string())?;
    let batches = write_legacy_office_document_extract_output(source, output.path()).await?;
    write_arrow_file(
        output
            .path()
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .as_path(),
        batches.as_slice(),
    )?;
    let restored = read_arrow_file(
        output
            .path()
            .join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME)
            .as_path(),
    )?;
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].num_rows(), 1);
    let resource_type = string_column(&restored[0], "resourceType")?;
    let content = string_column(&restored[0], "content")?;
    assert_eq!(resource_type.value(0), "legacy-office-document");
    assert!(!content.value(0).trim().is_empty());
    let report_path = output.path().join("_legacy_office_projection_report.json");
    let report = std::fs::read_to_string(report_path.as_path()).map_err(|error| {
        format!(
            "read legacy Office projection report `{}`: {error}",
            report_path.display()
        )
    })?;
    assert!(report.contains("\"precisionGatePassed\": true"));
    assert!(report.contains("\"sourceSha256\""));
    assert!(report.contains("\"cacheStatus\""));
    assert!(report.contains("\"lineCount\""));
    assert!(report.contains("\"markdownFencedBlockCount\""));
    let report_json: serde_json::Value = serde_json::from_str(report.as_str())
        .map_err(|error| format!("parse legacy Office projection report: {error}"))?;
    let cache_status = string_report_field(&report_json, "cacheStatus");
    if std::env::var_os("WENDAO_ARTIFACT_CACHE_BACKEND").is_some() {
        assert_ne!(cache_status, "Disabled");
    }
    Ok(LegacyOfficeDiagnosticResult {
        format: string_report_field(&report_json, "format").to_string(),
        rows: restored[0].num_rows(),
        content_chars: content.value(0).chars().count(),
        line_count: u64_report_field(&report_json, "lineCount"),
        tab_delimited_row_count: u64_report_field(&report_json, "tabDelimitedRowCount"),
        max_column_count: u64_report_field(&report_json, "maxColumnCount"),
        markdown_fenced_block_count: u64_report_field(&report_json, "markdownFencedBlockCount"),
        cache_backend: string_report_field(&report_json, "cacheBackend").to_string(),
        cache_status: cache_status.to_string(),
    })
}

fn collect_legacy_office_sources(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    collect_legacy_office_sources_inner(root, &mut sources)?;
    sources.sort();
    Ok(sources)
}

fn collect_legacy_office_sources_inner(
    root: &Path,
    sources: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|error| {
        format!(
            "read legacy Office diagnostic root `{}`: {error}",
            root.display()
        )
    })? {
        let entry =
            entry.map_err(|error| format!("read legacy Office diagnostic entry: {error}"))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| format!("read legacy Office diagnostic metadata: {error}"))?;
        if metadata.is_dir() {
            collect_legacy_office_sources_inner(path.as_path(), sources)?;
        } else if metadata.is_file() && is_legacy_office_source(path.as_path()) {
            sources.push(path);
        }
    }
    Ok(())
}

fn string_report_field<'a>(report: &'a serde_json::Value, field: &str) -> &'a str {
    report
        .get(field)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing")
}

fn u64_report_field(report: &serde_json::Value, field: &str) -> u64 {
    report
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

fn string_column<'a>(
    batch: &'a arrow::record_batch::RecordBatch,
    name: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("`{name}` column is not Utf8"))
}

fn sample_projection(format: LegacyOfficeFormat) -> LegacyOfficeProjection {
    let text = "alpha".to_string();
    let markdown = "# alpha\n".to_string();
    LegacyOfficeProjection {
        format,
        quality_metrics: legacy_office_quality_metrics(format, text.as_str(), markdown.as_str()),
        text,
        markdown,
    }
}
