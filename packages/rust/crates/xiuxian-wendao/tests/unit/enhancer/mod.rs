use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::link_graph_refs::LinkGraphEntityRef;

use crate::enhancer::{NoteInput, enhance_note, enhance_notes_batch, infer_relations};

fn read_json_snapshot(relative: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(relative);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read snapshot {}: {error}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse snapshot {}: {error}", path.display()))
}

#[test]
fn test_infer_relations_from_wikilinks_are_structural() {
    let refs = vec![LinkGraphEntityRef::new(
        "Python".to_string(),
        None,
        "[[Python]]".to_string(),
    )];
    let relations = infer_relations("docs/test.md", "Test Doc", "Content", &refs);

    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].source, "Test Doc");
    assert_eq!(relations[0].source_address, None);
    assert_eq!(relations[0].target, "Python");
    assert_eq!(relations[0].target_address, None);
    assert_eq!(relations[0].relation_type, None);
    assert_eq!(relations[0].metadata_owner, None);
}

#[test]
fn test_infer_relations_do_not_invent_semantics_from_skill_path() {
    let relations = infer_relations("assets/skills/git/SKILL.md", "Git Skill", "Content", &[]);

    assert!(relations.is_empty());
}

#[test]
fn test_infer_relations_do_not_promote_frontmatter_tags_to_relations() {
    let content = "---\ntags:\n  - search\n  - vector\n---\nBody";
    let relations = infer_relations("docs/test.md", "Test", content, &[]);

    assert!(relations.is_empty());
}

#[test]
fn test_infer_relations_from_property_drawers_are_scoped_and_explicit() {
    let content = r"
## Heading 1
:PROPERTIES:
:ID: heading-1
:RELATED: [[file-b#section-2]], [[#local-target]], [[/Heading 2]]
:WEIGHT: 5
:END:
";

    let relations = infer_relations("docs/a.md", "Doc A", content, &[]);

    assert_eq!(relations.len(), 3);

    assert_eq!(relations[0].source, "Doc A");
    assert_eq!(relations[0].source_address.as_deref(), Some("#heading-1"));
    assert_eq!(relations[0].target, "file-b");
    assert_eq!(relations[0].target_address.as_deref(), Some("#section-2"));
    assert_eq!(relations[0].relation_type.as_deref(), Some("RELATED_TO"));
    assert_eq!(relations[0].metadata_owner.as_deref(), Some("RELATED"));

    assert_eq!(relations[1].target, "Doc A");
    assert_eq!(
        relations[1].target_address.as_deref(),
        Some("#local-target")
    );

    assert_eq!(relations[2].target, "Doc A");
    assert_eq!(relations[2].target_address.as_deref(), Some("/Heading 2"));
}

#[test]
fn test_enhance_note_full() {
    let input = NoteInput {
        path: "docs/test.md".to_string().into(),
        title: "Test Doc".to_string(),
        content: "---\ntitle: Test\ntags:\n  - demo\n---\nContent with [[Python#Syntax]] ref"
            .to_string(),
    };

    let result = enhance_note(&input);
    assert_eq!(result.frontmatter.title.as_deref(), Some("Test"));
    assert_eq!(result.entity_refs.len(), 1);
    assert_eq!(result.entity_refs[0].name, "Python");
    assert_eq!(
        result.entity_refs[0].target_address.as_deref(),
        Some("#Syntax")
    );
    assert!(result.ref_stats.total_refs >= 1);
    assert_eq!(result.inferred_relations.len(), 1);
    assert_eq!(result.inferred_relations[0].source, "Test Doc");
    assert_eq!(result.inferred_relations[0].source_address, None);
    assert_eq!(result.inferred_relations[0].target, "Python");
    assert_eq!(
        result.inferred_relations[0].target_address.as_deref(),
        Some("#Syntax")
    );
    assert_eq!(result.inferred_relations[0].relation_type, None);
    assert_eq!(result.inferred_relations[0].metadata_owner, None);
}

#[test]
fn test_enhance_notes_batch() {
    let inputs = vec![
        NoteInput {
            path: "a.md".to_string().into(),
            title: "A".to_string(),
            content: "About [[X]]".to_string(),
        },
        NoteInput {
            path: "b.md".to_string().into(),
            title: "B".to_string(),
            content: "About [[Y]] and [[Z]]".to_string(),
        },
    ];

    let results = enhance_notes_batch(&inputs);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].entity_refs.len(), 1);
    assert_eq!(results[1].entity_refs.len(), 2);
}

#[test]
fn test_enhance_note_reference_relations_snapshot() {
    let input = NoteInput {
        path: "snapshots/reference.md".to_string().into(),
        title: "Snapshot Persona".to_string(),
        content: r"---
title: Snapshot Persona
---
## Planning
:PROPERTIES:
:ID: planning
:RELATED: #policy-anchor
:SEE_ALSO: /Appendix
:WEIGHT: 5
:END:

Global body references [[rules#Reference]] and [[agenda_flow.toml#^flow-block]].

## Appendix
:PROPERTIES:
:ID: policy-anchor
:END:
"
        .to_string(),
    };

    let result = enhance_note(&input);
    let payload = json!({
        "entity_refs": result.entity_refs,
        "inferred_relations": result.inferred_relations,
    });
    let expected = read_json_snapshot("parser/markdown/reference_relations.json");

    assert_eq!(payload, expected);
}
