use super::{assemble, temp_dir, write_main_file};

#[test]
fn test_assemble_skill_special_characters() {
    let temp_dir = temp_dir();
    let special_content = r#"# Special Chars

| Column | Value |
|--------|-------|
| Name    | {{ name }} |

```python
def hello():
    print("{{ name }}")
```

- [ ] Task 1
- [x] Task 2
"#;
    let main_path = write_main_file(&temp_dir, special_content);

    let result = assemble(main_path, Vec::new(), serde_json::json!({"name": "World"}));

    assert!(result.content.contains("Special Chars"));
    assert!(result.content.contains("World"));
    assert!(result.token_count > 0);
}

#[test]
fn test_assemble_skill_preserves_markdown_structure() {
    let temp_dir = temp_dir();
    let content = r"# Title

## Section 1

Content 1

## Section 2

Content 2

### Subsection

More content
";
    let main_path = write_main_file(&temp_dir, content);

    let result = assemble(main_path, Vec::new(), serde_json::json!({}));

    assert!(result.content.contains("# Title"));
    assert!(result.content.contains("## Section 1"));
    assert!(result.content.contains("## Section 2"));
    assert!(result.content.contains("### Subsection"));
}

#[test]
fn test_assemble_skill_nested_variables() {
    let temp_dir = temp_dir();
    let content = r"---
name: {{ skill.name }}
version: {{ skill.version }}
description: {{ skill.description }}
---

# {{ skill.name }}

This is version {{ skill.version }}.
";
    let main_path = write_main_file(&temp_dir, content);

    let result = assemble(
        main_path,
        Vec::new(),
        serde_json::json!({
            "skill": {
                "name": "TestSkill",
                "version": "1.0.0",
                "description": "A test skill"
            }
        }),
    );

    assert!(result.content.contains("name: TestSkill"));
    assert!(result.content.contains("version: 1.0.0"));
    assert!(result.content.contains("description: A test skill"));
    assert!(result.content.contains("# TestSkill"));
    assert!(result.content.contains("This is version 1.0.0."));
}
