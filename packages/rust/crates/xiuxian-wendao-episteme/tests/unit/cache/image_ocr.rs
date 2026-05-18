use xiuxian_wendao_episteme::write_image_ocr_cache_outputs;

use super::{image_task, sha256_bytes, single_pixel_png_bytes};

#[test]
fn image_ocr_cache_materializer_writes_review_blocked_outputs() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("images")).map_err(|error| error.to_string())?;
    let image_bytes = single_pixel_png_bytes();
    let image_path = corpus_root.join("images/evidence.png");
    std::fs::write(&image_path, image_bytes).map_err(|error| error.to_string())?;
    let source_hash = sha256_bytes(&image_bytes);
    let results_path = run_dir.join("ocr_results.jsonl");
    std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.image.001","text":"# OCR markdown","ocr_engine":"hosted-vlm-openai-compatible","ocr_profile":"hosted-vlm-direct-ocr-v1","text_mime_type":"text/markdown","source_sha256":"{source_hash}"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![image_task(
        "synthetic.extract.image.001",
        "images/evidence.png",
        source_hash,
        "outputs/synthetic.extract.image.001.json",
    )];

    let report = write_image_ocr_cache_outputs(&tasks, &results_path, &run_dir, &corpus_root)
        .map_err(|error| error.to_string())?;

    assert!(report.passed);
    assert_eq!(report.succeeded_count, 1);
    assert_eq!(report.failed_count, 0);
    let output_path = run_dir.join("outputs/synthetic.extract.image.001.json");
    let output = std::fs::read_to_string(output_path).map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(
        payload["schema_version"],
        "xiuxian_wendao.episteme_evidence_text_cache.v1"
    );
    assert_eq!(payload["status"], "succeeded");
    assert_eq!(payload["ocr_executed"], true);
    assert_eq!(payload["review_status"], "review_required");
    assert_eq!(payload["promotion_status"], "blocked_pending_review");
    assert_eq!(payload["ontology_truth"], false);
    assert_eq!(payload["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(payload["image_width"], 1);
    assert_eq!(payload["image_height"], 1);
    Ok(())
}

#[test]
fn image_ocr_cache_materializer_rejects_output_path_escape() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("images")).map_err(|error| error.to_string())?;
    let image_bytes = single_pixel_png_bytes();
    std::fs::write(corpus_root.join("images/evidence.png"), image_bytes)
        .map_err(|error| error.to_string())?;
    let source_hash = sha256_bytes(&image_bytes);
    let results_path = run_dir.join("ocr_results.jsonl");
    std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.image.escape","text":"# OCR markdown","source_sha256":"{source_hash}"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![image_task(
        "synthetic.extract.image.escape",
        "images/evidence.png",
        source_hash,
        "../escape.json",
    )];

    let error = write_image_ocr_cache_outputs(&tasks, &results_path, &run_dir, &corpus_root)
        .expect_err("output path escape should be rejected")
        .to_string();

    assert!(error.contains("clean relative path"));
    assert!(!temp.path().join("escape.json").exists());
    Ok(())
}

#[test]
fn image_ocr_cache_materializer_marks_source_path_escape_failed() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(&corpus_root).map_err(|error| error.to_string())?;
    let outside_image = temp.path().join("outside.png");
    let image_bytes = single_pixel_png_bytes();
    std::fs::write(&outside_image, image_bytes).map_err(|error| error.to_string())?;
    let source_hash = sha256_bytes(&image_bytes);
    let results_path = run_dir.join("ocr_results.jsonl");
    std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
    std::fs::write(
        &results_path,
        format!(
            r##"{{"queue_id":"synthetic.extract.image.source_escape","text":"# OCR markdown","source_sha256":"{source_hash}"}}"##
        ),
    )
    .map_err(|error| error.to_string())?;
    let tasks = vec![image_task(
        "synthetic.extract.image.source_escape",
        "../outside.png",
        source_hash,
        "outputs/synthetic.extract.image.source_escape.json",
    )];

    let report = write_image_ocr_cache_outputs(&tasks, &results_path, &run_dir, &corpus_root)
        .map_err(|error| error.to_string())?;

    assert!(!report.passed);
    assert_eq!(report.succeeded_count, 0);
    assert_eq!(report.failed_count, 1);
    let output =
        std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.image.source_escape.json"))
            .map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(payload["status"], "failed");
    assert!(
        payload["error"]
            .as_str()
            .unwrap_or_default()
            .contains("clean relative path")
    );
    Ok(())
}
