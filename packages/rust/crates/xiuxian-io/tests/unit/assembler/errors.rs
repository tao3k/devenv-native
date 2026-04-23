use crate::ContextAssembler;

use super::{assemble, temp_dir, write_main_file};

#[test]
fn test_assemble_skill_template_error_fallback() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Value: {{ undefined_var }}");

    let result = assemble(main_path, Vec::new(), serde_json::json!({}));

    assert!(result.content.contains("Template Error"));
}

#[test]
fn test_assemble_skill_missing_main_file() {
    let temp_dir = temp_dir();
    let missing_main = temp_dir.path().join("nonexistent.md");

    let error =
        match ContextAssembler::assemble_skill(missing_main, Vec::new(), serde_json::json!({})) {
            Ok(_value) => panic!("missing main file should return IoError::NotFound"),
            Err(error) => error,
        };

    assert!(error.to_string().contains("File not found"));
}
