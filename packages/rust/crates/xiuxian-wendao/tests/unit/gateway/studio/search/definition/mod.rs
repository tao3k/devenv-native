use std::path::Path;

use crate::gateway::studio::search::{DefinitionResolveOptions, resolve_best_definition};
use crate::gateway::studio::types::StudioNavigationTarget;

fn ast_hit(name: &str) -> crate::gateway::studio::types::AstSearchHit {
    crate::gateway::studio::types::AstSearchHit {
        name: name.to_string(),
        signature: format!("fn {name}()"),
        path: "src/lib.rs".to_string(),
        language: "rust".to_string(),
        crate_name: "demo".to_string(),
        project_name: None,
        root_label: None,
        node_kind: Some("function".to_string()),
        owner_title: None,
        navigation_target: StudioNavigationTarget {
            path: "src/lib.rs".to_string(),
            category: "symbol".to_string(),
            project_name: None,
            root_label: None,
            line: Some(10),
            line_end: Some(12),
            column: Some(1),
        },
        line_start: 10,
        line_end: 12,
        score: 1.0,
    }
}

#[test]
fn resolve_best_definition_uses_lexical_fallback_for_typos() {
    let hits = vec![ast_hit("spawn_local")];

    let result = resolve_best_definition(
        "spwan_local",
        hits.as_slice(),
        Path::new("."),
        Path::new("."),
        &[],
        &DefinitionResolveOptions::default(),
    )
    .unwrap_or_else(|| panic!("definition should resolve through fuzzy fallback"));

    assert_eq!(result.name, "spawn_local");
    assert!(result.score < 1.0);
    assert!(result.score > 0.0);
}
