use super::{assemble, temp_dir, write_main_file};

#[test]
fn test_assemble_skill_basic() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Hello {{ name }}!");

    let result = assemble(main_path, Vec::new(), serde_json::json!({"name": "World"}));

    assert!(result.content.contains("Hello World!"));
    assert!(result.token_count > 0);
    assert!(result.missing_refs.is_empty());
}

#[test]
fn test_assemble_skill_empty_variables() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "No variables here");

    let result = assemble(main_path, Vec::new(), serde_json::json!({}));

    assert!(result.content.contains("No variables here"));
    assert!(result.token_count > 0);
}

#[test]
fn test_assemble_skill_token_count_reasonable() {
    let temp_dir = temp_dir();
    let content = "word ".repeat(100);
    let main_path = write_main_file(&temp_dir, &content);

    let result = assemble(main_path, Vec::new(), serde_json::json!({}));

    assert!(result.token_count >= 20);
    assert!(result.token_count <= 150);
}

#[test]
fn test_assemble_result_fields() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Test {{ value }}");

    let result = assemble(
        main_path,
        Vec::new(),
        serde_json::json!({"value": "RESULT"}),
    );

    assert_eq!(result.content, "# Active Protocol\nTest RESULT");
    assert!(result.token_count > 0);
    assert!(result.missing_refs.is_empty());
}

#[test]
fn test_assemble_skill_no_references_section_when_empty() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "No references here");

    let result = assemble(main_path, Vec::new(), serde_json::json!({}));

    assert!(!result.content.contains("# Required References"));
}
