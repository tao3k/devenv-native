use xiuxian_wendao_episteme::write_docling_document_cache_outputs;

use super::docling_task;

#[test]
fn docling_document_cache_materializer_writes_review_blocked_outputs() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path();
    let results_path = run_dir.join("document_results.jsonl");
    std::fs::write(
        &results_path,
        r##"{"queue_id":"synthetic.extract.document.001","text":"# Docling markdown","extractor":"docling","docling_profile":"full","text_mime_type":"text/markdown","source_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","extension":"docx"}"##,
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![docling_task(
        "synthetic.extract.document.001",
        "outputs/synthetic.extract.document.001.json",
    )];

    let report = write_docling_document_cache_outputs(&tasks, &results_path, run_dir)
        .map_err(|error| error.to_string())?;

    assert!(report.passed);
    assert_eq!(report.succeeded_count, 1);
    assert_eq!(report.failed_count, 0);
    let output_path = run_dir.join("outputs/synthetic.extract.document.001.json");
    let output = std::fs::read_to_string(output_path).map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(
        payload["schema_version"],
        "xiuxian_wendao.episteme_evidence_text_cache.v1"
    );
    assert_eq!(payload["status"], "succeeded");
    assert_eq!(payload["extractor"], "docling");
    assert_eq!(payload["review_status"], "review_required");
    assert_eq!(payload["promotion_status"], "blocked_pending_review");
    assert_eq!(payload["ontology_truth"], false);
    assert_eq!(payload["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(payload["docling_document_executed"], true);
    Ok(())
}

#[test]
fn docling_document_cache_materializer_rejects_output_path_escape() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path();
    let results_path = run_dir.join("document_results.jsonl");
    std::fs::write(
        &results_path,
        r##"{"queue_id":"synthetic.extract.document.escape","text":"# Docling markdown","source_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","extension":"docx"}"##,
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![docling_task(
        "synthetic.extract.document.escape",
        "../escape.json",
    )];

    let error = write_docling_document_cache_outputs(&tasks, &results_path, run_dir)
        .expect_err("output path escape should be rejected")
        .to_string();

    assert!(error.contains("clean relative path"));
    assert!(!temp.path().join("escape.json").exists());
    Ok(())
}
