use include_dir::{Dir, include_dir};
use serde_json::json;
use xiuxian_qianji::{BootcampRunOptions, BootcampVfsMount};

pub(crate) const AGENDA_FLOW_URI_FROM_ALIAS: &str =
    "wendao://skills/agenda-lab/references/agenda_flow.toml";
pub(crate) const AGENDA_FLOW_URI_CANONICAL: &str =
    "wendao://skills/agenda-management/references/agenda_flow.toml";
pub(crate) const FORGE_FLOW_URI_CANONICAL: &str =
    "wendao://skills/forge-evolution/references/soul_forge_flow.toml";
pub(crate) const AGENDA_FACTS: &str = "timeboxing; execution order; deadline awareness; review loop; tool output fidelity; single message clarity; language alignment; cognitive load; historical carryover; execution realism; risk-first review; carryover=1; milimeter-level alignment; audit trail; traceability; architectural consistency";

pub(crate) static ZHIXING_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../xiuxian-wendao/resources");
pub(crate) static AGENDA_OVERRIDE_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/agenda_override/resources");

pub(crate) fn zhixing_mount(
    semantic_name: &'static str,
    references_dir: &'static str,
) -> [BootcampVfsMount; 1] {
    [BootcampVfsMount::new(
        semantic_name,
        references_dir,
        &ZHIXING_RESOURCES,
    )]
}

pub(crate) fn runtime_default_llm_options() -> BootcampRunOptions {
    BootcampRunOptions::default()
}

pub(crate) fn mock_llm_options(_response: &str) -> BootcampRunOptions {
    BootcampRunOptions::default()
}

pub(crate) fn bootcamp_context_from_env() -> Option<serde_json::Value> {
    let Ok(context_raw) = std::env::var("XIUXIAN_BOOTCAMP_CONTEXT") else {
        return None;
    };
    let context = serde_json::from_str(&context_raw)
        .unwrap_or_else(|error| panic!("Failed to parse XIUXIAN_BOOTCAMP_CONTEXT: {error}"));
    Some(context)
}

pub(crate) fn context_object_mut(
    context: &mut serde_json::Value,
) -> &mut serde_json::Map<String, serde_json::Value> {
    let Some(context_object) = context.as_object_mut() else {
        panic!("XIUXIAN_BOOTCAMP_CONTEXT must be a JSON object");
    };
    context_object
}

pub(crate) fn ensure_runtime_forge_context_defaults(context: &mut serde_json::Value) {
    use tempfile::tempdir;

    let workspace = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let context_object = context_object_mut(context);
    context_object
        .entry("project_root".to_string())
        .or_insert_with(|| json!(workspace.path().display().to_string()));
    context_object
        .entry("target_persona_dir".to_string())
        .or_insert_with(|| json!(workspace.path().join("personas").display().to_string()));
    context_object
        .entry("role_id".to_string())
        .or_insert_with(|| json!("runtime_soul_forger"));
}

pub(crate) fn result_error<T, E>(result: Result<T, E>, message: &str) -> E {
    match result {
        Ok(_) => panic!("{message}"),
        Err(error) => error,
    }
}
