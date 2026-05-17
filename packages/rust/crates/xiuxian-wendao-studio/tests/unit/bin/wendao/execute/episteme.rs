use std::path::Path;

use super::{
    absolute_runtime_path, image_ocr_analyzer_command_spec, image_ocr_cache_bridge_command_spec,
};

fn expected_args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

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
fn image_ocr_cache_bridge_command_spec_preserves_no_promotion_bridge() {
    let spec = image_ocr_cache_bridge_command_spec(
        "python",
        Path::new("/episteme"),
        Path::new("/episteme/tools/run_extraction_plan.py"),
        Path::new("/episteme/runs/extraction/seed/tasks.tsv"),
        Path::new("/corpus"),
        Path::new("/episteme/runs/extraction/seed/ocr_results.jsonl"),
    );

    assert_eq!(spec.program, "python");
    assert_eq!(spec.current_dir.as_deref(), Some("/episteme"));
    assert_eq!(
        spec.args,
        expected_args(&[
            "/episteme/tools/run_extraction_plan.py",
            "--corpus-root",
            "/corpus",
            "--tasks",
            "/episteme/runs/extraction/seed/tasks.tsv",
            "--ocr-results-jsonl",
            "/episteme/runs/extraction/seed/ocr_results.jsonl",
        ])
    );
}

#[test]
fn image_ocr_command_paths_are_absolute_for_episteme_current_dir() -> Result<(), String> {
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
