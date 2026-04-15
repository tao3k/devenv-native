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
