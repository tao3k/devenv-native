use xiuxian_wendao_episteme::{
    EpistemeLegacyOfficeConversionRequest, convert_legacy_office_tasks,
    read_legacy_office_conversion_tasks_tsv, validate_docling_document_tasks,
    validate_legacy_office_conversion_tasks,
};

use super::{legacy_office_task, sha256_bytes};

#[test]
fn legacy_office_conversion_tasks_accept_only_legacy_route() -> Result<(), String> {
    let source_hash = sha256_bytes(b"legacy office bytes");
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.doc",
        "docs/evidence.doc",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.doc.json",
    )];

    validate_legacy_office_conversion_tasks(&tasks).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn legacy_office_conversion_tasks_reject_docling_route() -> Result<(), String> {
    let source_hash = sha256_bytes(b"legacy office bytes");
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.doc",
        "docs/evidence.doc",
        source_hash,
        "document_text_evidence",
        "outputs/synthetic.extract.legacy.doc.json",
    )];

    let Err(error) = validate_legacy_office_conversion_tasks(&tasks) else {
        return Err("legacy conversion should reject Docling route".to_string());
    };
    assert!(
        error
            .to_string()
            .contains("legacy_office_document_evidence")
    );
    Ok(())
}

#[test]
fn legacy_office_conversion_tasks_reject_modern_document_extension() -> Result<(), String> {
    let source_hash = sha256_bytes(b"modern document bytes");
    let tasks = vec![legacy_office_task(
        "synthetic.extract.modern.docx",
        "docs/evidence.docx",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.modern.docx.json",
    )];

    let Err(error) = validate_legacy_office_conversion_tasks(&tasks) else {
        return Err("legacy conversion should reject modern document extensions".to_string());
    };
    assert!(error.to_string().contains("doc/ppt/xls"));
    Ok(())
}

#[test]
fn docling_document_tasks_reject_legacy_office_conversion_tasks() -> Result<(), String> {
    let source_hash = sha256_bytes(b"legacy office bytes");
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.xls",
        "docs/evidence.xls",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.xls.json",
    )];

    let Err(error) = validate_docling_document_tasks(&tasks) else {
        return Err("Docling document validator should reject legacy Office tasks".to_string());
    };
    assert!(error.to_string().contains("unsupported document tasks"));
    Ok(())
}

#[test]
fn legacy_office_conversion_tasks_tsv_rejects_duplicate_queue_ids() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let tasks_path = temp.path().join("tasks.tsv");
    std::fs::write(
        &tasks_path,
        "queue_id\tfile_id\trelative_path\tcategory\tlanguage\textraction_route\tpriority\tsource_sha256\tplanned_output_path\toutput_contract\tstatus\n\
synthetic.duplicate\tfile.1\tdocs/a.doc\tsynthetic\tzh-CN\tlegacy_office_document_evidence\t10\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\toutputs/a.json\tcache_only_no_rdf_promotion\tplanned\n\
synthetic.duplicate\tfile.2\tdocs/b.xls\tsynthetic\tzh-CN\tlegacy_office_document_evidence\t10\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\toutputs/b.json\tcache_only_no_rdf_promotion\tplanned\n",
    )
    .map_err(|error| error.to_string())?;

    let Err(error) = read_legacy_office_conversion_tasks_tsv(&tasks_path) else {
        return Err("duplicate queue_id should be rejected".to_string());
    };

    assert!(error.to_string().contains("duplicate queue_id"));
    Ok(())
}

#[test]
fn legacy_office_conversion_runner_writes_dry_run_receipt() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"legacy office bytes";
    let source_hash = sha256_bytes(source_bytes);
    std::fs::write(corpus_root.join("docs/evidence.doc"), source_bytes)
        .map_err(|error| error.to_string())?;
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.doc",
        "docs/evidence.doc",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.doc.json",
    )];
    let request = EpistemeLegacyOfficeConversionRequest::new(temp.path().join("missing-converter"))
        .with_dry_run(true);

    let report = convert_legacy_office_tasks(&tasks, &run_dir, &corpus_root, &request)
        .map_err(|error| error.to_string())?;

    assert!(report.skipped);
    assert!(report.passed);
    assert_eq!(report.skipped_count, 1);
    let output = std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.legacy.doc.json"))
        .map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(payload["status"], "skipped");
    assert_eq!(payload["conversion_executed"], false);
    assert_eq!(payload["raw_to_rdf_promotion_allowed"], false);
    assert_eq!(payload["converted_extension"], "docx");
    assert!(
        !run_dir
            .join("outputs/converted/synthetic.extract.legacy.doc.docx")
            .exists()
    );
    assert!(
        run_dir
            .join("legacy_office_conversion_receipt.json")
            .is_file()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn legacy_office_conversion_runner_executes_fake_converter() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"legacy office bytes";
    let source_hash = sha256_bytes(source_bytes);
    std::fs::write(corpus_root.join("docs/evidence.xls"), source_bytes)
        .map_err(|error| error.to_string())?;
    let converter_path = write_fake_converter(temp.path(), "cp \"$1\" \"$2\"\n")?;
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.xls",
        "docs/evidence.xls",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.xls.json",
    )];
    let request = EpistemeLegacyOfficeConversionRequest::new(converter_path);

    let report = convert_legacy_office_tasks(&tasks, &run_dir, &corpus_root, &request)
        .map_err(|error| error.to_string())?;

    assert!(!report.skipped);
    assert!(report.passed);
    assert_eq!(report.succeeded_count, 1);
    let converted_path = run_dir.join("outputs/converted/synthetic.extract.legacy.xls.xlsx");
    assert_eq!(
        std::fs::read(&converted_path).map_err(|error| error.to_string())?,
        source_bytes
    );
    let output = std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.legacy.xls.json"))
        .map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(payload["status"], "succeeded");
    assert_eq!(payload["converted_extension"], "xlsx");
    assert_eq!(payload["conversion_executed"], true);
    assert_eq!(payload["ontology_truth"], false);
    Ok(())
}

#[test]
fn legacy_office_conversion_runner_records_source_hash_drift() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    std::fs::write(corpus_root.join("docs/evidence.ppt"), b"changed bytes")
        .map_err(|error| error.to_string())?;
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.ppt",
        "docs/evidence.ppt",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.ppt.json",
    )];
    let request = EpistemeLegacyOfficeConversionRequest::new(temp.path().join("missing-converter"));

    let report = convert_legacy_office_tasks(&tasks, &run_dir, &corpus_root, &request)
        .map_err(|error| error.to_string())?;

    assert!(!report.passed);
    assert_eq!(report.failed_count, 1);
    let output = std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.legacy.ppt.json"))
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

#[cfg(unix)]
#[test]
fn legacy_office_conversion_runner_records_missing_converter_output() -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let run_dir = temp.path().join("run");
    let corpus_root = temp.path().join("corpus");
    std::fs::create_dir_all(corpus_root.join("docs")).map_err(|error| error.to_string())?;
    let source_bytes = b"legacy office bytes";
    let source_hash = sha256_bytes(source_bytes);
    std::fs::write(corpus_root.join("docs/evidence.doc"), source_bytes)
        .map_err(|error| error.to_string())?;
    let converter_path = write_fake_converter(temp.path(), "exit 0\n")?;
    let tasks = vec![legacy_office_task(
        "synthetic.extract.legacy.doc",
        "docs/evidence.doc",
        source_hash,
        "legacy_office_document_evidence",
        "outputs/synthetic.extract.legacy.doc.json",
    )];
    let request = EpistemeLegacyOfficeConversionRequest::new(converter_path);

    let report = convert_legacy_office_tasks(&tasks, &run_dir, &corpus_root, &request)
        .map_err(|error| error.to_string())?;

    assert!(!report.passed);
    assert_eq!(report.failed_count, 1);
    let output = std::fs::read_to_string(run_dir.join("outputs/synthetic.extract.legacy.doc.json"))
        .map_err(|error| error.to_string())?;
    let payload: serde_json::Value =
        serde_json::from_str(&output).map_err(|error| error.to_string())?;
    assert_eq!(payload["status"], "failed");
    assert_eq!(payload["source_hash_matched"], true);
    assert!(
        payload["error"]
            .as_str()
            .unwrap_or_default()
            .contains("did not produce")
    );
    Ok(())
}

#[cfg(unix)]
fn write_fake_converter(root: &std::path::Path, body: &str) -> Result<std::path::PathBuf, String> {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join("fake-converter.sh");
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).map_err(|error| error.to_string())?;
    let mut permissions = std::fs::metadata(&path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).map_err(|error| error.to_string())?;
    Ok(path)
}
