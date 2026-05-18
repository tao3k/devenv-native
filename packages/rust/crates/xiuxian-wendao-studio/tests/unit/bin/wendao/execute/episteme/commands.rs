use super::{
    Path, absolute_runtime_path, docling_document_analyzer_command_spec, expected_args,
    image_ocr_analyzer_command_spec,
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
