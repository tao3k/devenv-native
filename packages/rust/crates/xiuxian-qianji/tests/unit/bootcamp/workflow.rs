use super::super::manifest::{parse_manifest, parsed_manifest_requires_link_graph};
use super::inject_llm_model_fallback_if_missing;
use serde_json::json;

#[test]
fn injects_llm_model_fallback_when_missing() {
    let mut context = json!({
        "request": "Critique this agenda."
    });
    inject_llm_model_fallback_if_missing(&mut context, "mimo-v2-pro");
    assert_eq!(context["llm_model_fallback"], json!("mimo-v2-pro"));
}

#[test]
fn preserves_existing_explicit_llm_model() {
    let mut context = json!({
        "llm_model": "override-model"
    });
    inject_llm_model_fallback_if_missing(&mut context, "mimo-v2-pro");
    assert!(context.get("llm_model_fallback").is_none());
    assert_eq!(context["llm_model"], json!("override-model"));
}

#[test]
fn preserves_existing_llm_model_fallback() {
    let mut context = json!({
        "llm_model_fallback": "preset-model"
    });
    inject_llm_model_fallback_if_missing(&mut context, "mimo-v2-pro");
    assert_eq!(context["llm_model_fallback"], json!("preset-model"));
}

#[test]
fn manifest_without_knowledge_nodes_skips_link_graph_requirement() {
    let manifest = parse_manifest(
        r#"
name = "AnnotationOnly"

[[nodes]]
id = "annotate"
task_type = "annotation"
weight = 1.0
params = { prompt = "noop" }
"#,
    )
    .unwrap_or_else(|error| panic!("manifest should parse: {error}"));

    assert!(!parsed_manifest_requires_link_graph(&manifest));
}

#[test]
fn manifest_with_knowledge_nodes_requires_link_graph() {
    let manifest = parse_manifest(
        r#"
name = "KnowledgeOnly"

[[nodes]]
id = "search"
task_type = "knowledge"
weight = 1.0
params = {}
"#,
    )
    .unwrap_or_else(|error| panic!("manifest should parse: {error}"));

    assert!(parsed_manifest_requires_link_graph(&manifest));
}
