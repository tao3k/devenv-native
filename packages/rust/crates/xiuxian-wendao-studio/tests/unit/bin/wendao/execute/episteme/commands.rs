use super::{
    EpistemeRuntimeConfig, Path, absolute_runtime_path, docling_document_analyzer_command_spec,
    expected_args, image_ocr_analyzer_command_spec, resolve_legacy_office_converter,
    should_skip_analyzer,
};

#[test]
fn image_ocr_analyzer_command_spec_is_queue_keyed_jsonl() {
    let spec = image_ocr_analyzer_command_spec(
        "wendao-image-ocr-jsonl",
        Path::new("/episteme"),
        Path::new("/episteme/runs/extraction/seed/tasks.tsv"),
        Path::new("/corpus"),
        Path::new("/episteme/runs/extraction/seed/ocr_results.jsonl"),
    );

    assert_eq!(spec.program, "wendao-image-ocr-jsonl");
    assert_eq!(spec.current_dir.as_deref(), Some("/episteme"));
    assert_eq!(
        spec.args,
        expected_args(&[
            "--tasks",
            "/episteme/runs/extraction/seed/tasks.tsv",
            "--corpus-root",
            "/corpus",
            "--output-jsonl",
            "/episteme/runs/extraction/seed/ocr_results.jsonl",
        ])
    );
}

#[test]
fn docling_document_analyzer_command_spec_is_queue_keyed_jsonl() {
    let spec = docling_document_analyzer_command_spec(
        "wendao-docling-document-jsonl",
        Path::new("/episteme"),
        Path::new("/episteme/runs/extraction/seed/tasks.tsv"),
        Path::new("/corpus"),
        Path::new("/episteme/runs/extraction/seed/document_results.jsonl"),
        "full",
    );

    assert_eq!(spec.program, "wendao-docling-document-jsonl");
    assert_eq!(spec.current_dir.as_deref(), Some("/episteme"));
    assert_eq!(
        spec.args,
        expected_args(&[
            "--tasks",
            "/episteme/runs/extraction/seed/tasks.tsv",
            "--corpus-root",
            "/corpus",
            "--output-jsonl",
            "/episteme/runs/extraction/seed/document_results.jsonl",
            "--profile",
            "full",
        ])
    );
}

#[test]
fn command_paths_are_absolute_for_episteme_current_dir() -> Result<(), String> {
    let current_dir = std::env::current_dir().map_err(|error| error.to_string())?;

    assert_eq!(
        absolute_runtime_path(Path::new("episteme/runs/tasks.tsv"))
            .map_err(|error| error.to_string())?,
        current_dir.join("episteme/runs/tasks.tsv")
    );
    assert_eq!(
        absolute_runtime_path(Path::new("/episteme/runs/tasks.tsv"))
            .map_err(|error| error.to_string())?,
        Path::new("/episteme/runs/tasks.tsv")
    );
    Ok(())
}

#[test]
fn analyzer_execution_is_skipped_for_dry_run_or_existing_results() {
    assert!(should_skip_analyzer(true, false));
    assert!(should_skip_analyzer(false, true));
    assert!(should_skip_analyzer(true, true));
    assert!(!should_skip_analyzer(false, false));
}

#[test]
fn legacy_office_converter_prefers_explicit_path() -> Result<(), String> {
    let config = EpistemeRuntimeConfig {
        legacy_office_converter: Some("/configured/converter".into()),
        ..EpistemeRuntimeConfig::default()
    };

    let resolved =
        resolve_legacy_office_converter(Some(&"/explicit/converter".into()), Some(&config), false)
            .map_err(|error| error.to_string())?;

    assert_eq!(resolved, Path::new("/explicit/converter"));
    Ok(())
}

#[test]
fn legacy_office_converter_uses_episteme_runtime_config() -> Result<(), String> {
    let config = EpistemeRuntimeConfig {
        legacy_office_converter: Some("/configured/converter".into()),
        ..EpistemeRuntimeConfig::default()
    };

    let resolved = resolve_legacy_office_converter(None, Some(&config), false)
        .map_err(|error| error.to_string())?;

    assert_eq!(resolved, Path::new("/configured/converter"));
    Ok(())
}

#[test]
fn legacy_office_converter_dry_run_uses_placeholder_without_config() -> Result<(), String> {
    let resolved =
        resolve_legacy_office_converter(None, None, true).map_err(|error| error.to_string())?;

    assert_eq!(resolved, Path::new("legacy-office-converter"));
    Ok(())
}

#[test]
fn legacy_office_converter_requires_config_for_execution() -> Result<(), String> {
    let Err(error) = resolve_legacy_office_converter(None, None, false) else {
        return Err("legacy Office execution should require converter config".to_string());
    };

    assert!(error.to_string().contains("legacy_office_converter"));
    Ok(())
}
