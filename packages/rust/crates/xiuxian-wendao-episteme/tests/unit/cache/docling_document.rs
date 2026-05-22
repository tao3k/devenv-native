use xiuxian_wendao_episteme::{
    read_docling_document_tasks_tsv, write_docling_document_cache_outputs,
};

use super::{docling_task, sha256_bytes};

#[test]
fn docling_document_cache_materializer_writes_review_blocked_outputs() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path();
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"synthetic docx bytes";
    let source_hash = sha256_bytes(source_bytes);
    std::fs::write(corpus_root.join("docs/evidence.docx"), source_bytes)
        .map_err(|error| error.to_string())?;
    let results_path = run_dir.join("document_results.jsonl");
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.document.001","text":"# Docling markdown","extractor":"docling","docling_profile":"full","text_mime_type":"text/markdown","source_sha256":"{source_hash}","extension":"docx"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![docling_task(
        "synthetic.extract.document.001",
        source_hash,
        "outputs/synthetic.extract.document.001.json",
    )];

    let report = write_docling_document_cache_outputs(&tasks, &results_path, run_dir, &corpus_root)
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
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"synthetic docx bytes";
    let source_hash = sha256_bytes(source_bytes);
    std::fs::write(corpus_root.join("docs/evidence.docx"), source_bytes)
        .map_err(|error| error.to_string())?;
    let results_path = run_dir.join("document_results.jsonl");
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.document.escape","text":"# Docling markdown","source_sha256":"{source_hash}","extension":"docx"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![docling_task(
        "synthetic.extract.document.escape",
        source_hash,
        "../escape.json",
    )];

    let Err(error) =
        write_docling_document_cache_outputs(&tasks, &results_path, run_dir, &corpus_root)
    else {
        return Err("output path escape should be rejected".to_string());
    };
    let error = error.to_string();

    assert!(error.contains("clean relative path"));
    assert!(!temp.path().join("escape.json").exists());
    Ok(())
}

#[test]
fn docling_document_cache_materializer_marks_source_hash_drift_failed() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path();
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"changed document bytes";
    std::fs::write(corpus_root.join("docs/evidence.docx"), source_bytes)
        .map_err(|error| error.to_string())?;
    let stale_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
    let results_path = run_dir.join("document_results.jsonl");
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.document.drift","text":"# Docling markdown","source_sha256":"{stale_hash}","extension":"docx"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![docling_task(
        "synthetic.extract.document.drift",
        stale_hash,
        "outputs/synthetic.extract.document.drift.json",
    )];

    let report = write_docling_document_cache_outputs(&tasks, &results_path, run_dir, &corpus_root)
        .map_err(|error| error.to_string())?;

    assert!(!report.passed);
    assert_eq!(report.succeeded_count, 0);
    assert_eq!(report.failed_count, 1);
    let output =
        std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.document.drift.json"))
            .map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(payload["status"], "failed");
    assert_eq!(payload["source_hash_matched"], false);
    assert!(
        payload["error"]
            .as_str()
            .unwrap_or_default()
            .contains("source sha256 drift")
    );
    Ok(())
}

#[test]
fn docling_document_tasks_tsv_rejects_duplicate_queue_ids() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tasks_path = temp.path().join("tasks.tsv");
    std::fs::write(
        &tasks_path,
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\tsource_sha256\tplanned_output_path\toutput_contract\tstatus\n\
synthetic.duplicate\tfile.1\tdocs/a.docx\tsynthetic\tzh-CN\tdocument_text_evidence\t10\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\toutputs/a.json\tcache_only_no_rdf_promotion\tplanned\n\
synthetic.duplicate\tfile.2\tdocs/b.docx\tsynthetic\tzh-CN\tdocument_text_evidence\t10\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\toutputs/b.json\tcache_only_no_rdf_promotion\tplanned\n",
    )
    .map_err(|error| error.to_string())?;

    let Err(error) = read_docling_document_tasks_tsv(&tasks_path) else {
        return Err("duplicate queue_id should be rejected".to_string());
    };

    assert!(error.to_string().contains("duplicate queue_id"));
    Ok(())
}
