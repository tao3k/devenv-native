use super::support::{
    Entity, EntityType, ManifestationManager, PersonaRegistry, build_manifestation_manager,
    extract_markdown_config_blocks, fs, persona_records, render_task_add_response, tempdir,
    template_records,
};

#[tokio::test]
async fn task_add_render_uses_hot_reloaded_manifestation_template()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let template_tmp = tempdir()?;
    let template_path = template_tmp.path().join("task_add_response.md");
    fs::write(&template_path, "Template v1 -> {{ task_title }}")?;

    let template_glob = format!("{}/*.md", template_tmp.path().display());
    let manifestation = ManifestationManager::new(&[template_glob.as_str()])?;

    let task_id = "task:hot-reload-confirmation";
    let task = Entity::new(
        task_id.to_string(),
        "Hot Reload Task".to_string(),
        EntityType::Other("Task".to_string()),
        "Verify manifestation template reload path".to_string(),
    );

    let first = render_task_add_response(&manifestation, &task)?;
    assert!(
        first.contains("Template v1 -> Hot Reload Task"),
        "expected v1 template output, got: {first}"
    );

    fs::write(&template_path, "Template v2 -> {{ task_title }}")?;
    let second = render_task_add_response(&manifestation, &task)?;
    assert!(
        second.contains("Template v2 -> Hot Reload Task"),
        "expected v2 template output without restart, got: {second}"
    );

    Ok(())
}

#[tokio::test]
async fn task_add_render_supports_markdown_ast_memory_bridge()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let markdown = r#"
## Persona: Agenda Steward
<!-- id: "agenda_steward", type: "persona" -->

```toml
name = "Agenda Steward"
voice_tone = "Structured and practical."
style_anchors = ["agenda", "clarity"]
cot_template = "Observe -> draft -> validate"
forbidden_words = ["impossible"]
```

## Template: Task Add Response
<!-- id: "task_add_response.j2", type: "template", target: "task_add_response.md" -->

```jinja2
Markdown Bridge Template -> {{ task_title }} :: {{ task_id }}
```
"#;

    let blocks = extract_markdown_config_blocks(markdown);

    let mut registry = PersonaRegistry::new();
    let loaded_personas = registry.load_from_memory_records(persona_records(&blocks))?;
    assert_eq!(loaded_personas, 1);
    assert!(registry.get("agenda_steward").is_some());

    let manifestation_manager = build_manifestation_manager()?;
    let loaded_templates =
        manifestation_manager.load_templates_from_memory(template_records(&blocks))?;
    assert_eq!(loaded_templates, 2);

    let task_id = "task:markdown-bridge";
    let task = Entity::new(
        task_id.to_string(),
        "Bridge Render Task".to_string(),
        EntityType::Other("Task".to_string()),
        "Verify markdown AST memory bridge".to_string(),
    );

    let rendered = render_task_add_response(&manifestation_manager, &task)?;
    assert!(rendered.contains("Markdown Bridge Template"));
    assert!(rendered.contains("Bridge Render Task"));
    assert!(rendered.contains(task_id));
    Ok(())
}
