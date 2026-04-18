use super::{assemble, temp_dir, write_file, write_main_file};

#[test]
fn test_assemble_skill_with_references() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Main: {{ var1 }}");
    let ref_path = write_file(&temp_dir, "ref.md", "Reference content");

    let result = assemble(
        main_path,
        vec![ref_path],
        serde_json::json!({"var1": "test"}),
    );

    assert!(result.content.contains("Main: test"));
    assert!(result.content.contains("Reference content"));
    assert!(result.content.contains("# Required References"));
}

#[test]
fn test_assemble_skill_missing_reference() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Main content");
    let missing_path = temp_dir.path().join("missing.md");

    let result = assemble(main_path, vec![missing_path], serde_json::json!({}));

    assert_eq!(result.missing_refs.len(), 1);
}

#[test]
fn test_assemble_skill_multiple_references() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Main with {{ ref1 }} and {{ ref2 }}");
    let ref1 = write_file(&temp_dir, "ref1.md", "Reference 1 content");
    let ref2 = write_file(&temp_dir, "ref2.md", "Reference 2 content");

    let result = assemble(
        main_path,
        vec![ref1, ref2],
        serde_json::json!({
            "ref1": "VAR1",
            "ref2": "VAR2"
        }),
    );

    assert!(result.content.contains("Main with VAR1 and VAR2"));
    assert!(result.content.contains("Reference 1 content"));
    assert!(result.content.contains("Reference 2 content"));
    assert!(result.missing_refs.is_empty());
}

#[test]
fn test_assemble_skill_all_missing_references() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Main content");
    let ref1 = temp_dir.path().join("missing1.md");
    let ref2 = temp_dir.path().join("missing2.md");

    let result = assemble(main_path, vec![ref1, ref2], serde_json::json!({}));

    assert_eq!(result.missing_refs.len(), 2);
}

#[test]
fn test_assemble_skill_reference_includes_filename() {
    let temp_dir = temp_dir();
    let main_path = write_main_file(&temp_dir, "Main content");
    let ref_path = write_file(&temp_dir, "my_reference.md", "Reference content");

    let result = assemble(main_path, vec![ref_path], serde_json::json!({}));

    assert!(result.content.contains("## my_reference.md"));
    assert!(result.content.contains("Reference content"));
}
