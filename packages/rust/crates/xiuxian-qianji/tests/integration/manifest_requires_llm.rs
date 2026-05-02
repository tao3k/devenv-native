//! Tests for manifest LLM requirement inspection.

use xiuxian_qianji::{manifest_declares_qianhuan_bindings, manifest_requires_llm};

#[test]
fn manifest_requires_llm_returns_false_for_non_llm_workflows() {
    let manifest = r#"
name = "NoLlm"

[[nodes]]
id = "Stage"
task_type = "command"
weight = 1.0
params = { cmd = "echo hi", output_key = "stdout" }

[[nodes]]
id = "Pause"
task_type = "suspend"
weight = 1.0
params = { reason = "wait", prompt = "Continue?", resume_key = "answer" }

[[edges]]
from = "Stage"
to = "Pause"
weight = 1.0
"#;

    assert!(
        !manifest_requires_llm(manifest)
            .unwrap_or_else(|error| panic!("manifest should parse and not require llm: {error}"))
    );
}

#[test]
fn manifest_requires_llm_returns_true_when_llm_task_exists() {
    let manifest = r#"
name = "HasLlm"

[[nodes]]
id = "Architect"
task_type = "llm"
weight = 1.0
params = { model = "MiniMax-M2.5", prompt = "Plan", output_key = "analysis" }
"#;

    assert!(
        manifest_requires_llm(manifest)
            .unwrap_or_else(|error| panic!("manifest should parse and require llm: {error}"))
    );
}

#[test]
fn manifest_requires_llm_returns_true_for_llm_augmented_formal_audit() {
    let manifest = r#"
name = "AugmentedAudit"

[[nodes]]
id = "StrictTeacher"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Steward"] }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
model = "MiniMax-M2.5"
"#;

    assert!(
        manifest_requires_llm(manifest).unwrap_or_else(|error| panic!(
            "augmented formal audit manifest should require llm: {error}"
        ))
    );
}

#[test]
fn manifest_requires_llm_returns_error_on_invalid_toml() {
    let result = manifest_requires_llm("this = ] invalid");
    assert!(result.is_err());
}

#[test]
fn manifest_declares_qianhuan_bindings_returns_true_when_present() {
    let manifest = r#"
name = "BindingManifest"

[[nodes]]
id = "Annotator"
task_type = "annotation"
weight = 1.0
params = {}
[nodes.qianhuan]
persona_id = "artisan-engineer"
template_target = "draft_reflection.md"
"#;

    assert!(
        manifest_declares_qianhuan_bindings(manifest).unwrap_or_else(|error| panic!(
            "manifest with qianhuan binding should parse successfully: {error}"
        ))
    );
}

#[test]
fn manifest_declares_qianhuan_bindings_returns_false_when_absent() {
    let manifest = r#"
name = "NoBindingManifest"

[[nodes]]
id = "Stage"
task_type = "command"
weight = 1.0
params = { cmd = "echo ok", output_key = "stdout" }
"#;

    assert!(
        !manifest_declares_qianhuan_bindings(manifest).unwrap_or_else(|error| panic!(
            "manifest without qianhuan binding should parse successfully: {error}"
        ))
    );
}

#[test]
fn manifest_parses_node_level_llm_binding_table() {
    let manifest = r#"
name = "LlmBindingManifest"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { prompt = "Plan", output_key = "analysis" }
[nodes.llm]
provider = "openai"
model = "gpt-4o-mini"
base_url = "http://tenant-a.local/v1"
api_key_env = "TENANT_A_API_KEY"
"#;

    let parsed: xiuxian_qianji::QianjiManifest = toml::from_str(manifest)
        .unwrap_or_else(|error| panic!("manifest with llm binding should parse: {error}"));
    let binding = parsed.nodes[0]
        .llm
        .as_ref()
        .unwrap_or_else(|| panic!("llm binding should be present"));

    assert_eq!(binding.provider.as_deref(), Some("openai"));
    assert_eq!(binding.model.as_deref(), Some("gpt-4o-mini"));
    assert_eq!(
        binding.base_url.as_deref(),
        Some("http://tenant-a.local/v1")
    );
    assert_eq!(binding.api_key_env.as_deref(), Some("TENANT_A_API_KEY"));
}

#[test]
fn manifest_parses_legacy_node_level_llm_config_alias() {
    let manifest = r#"
name = "LegacyLlmConfigAlias"

[[nodes]]
id = "Analyzer"
task_type = "llm"
weight = 1.0
params = { prompt = "Plan", output_key = "analysis" }
[nodes.llm_config]
provider = "openai"
model = "gpt-4o-mini"
base_url = "http://tenant-a.local/v1"
api_key_env = "TENANT_A_API_KEY"
"#;

    let parsed: xiuxian_qianji::QianjiManifest = toml::from_str(manifest)
        .unwrap_or_else(|error| panic!("manifest with llm_config alias should parse: {error}"));
    let binding = parsed.nodes[0]
        .llm
        .as_ref()
        .unwrap_or_else(|| panic!("llm binding should be present via llm_config alias"));

    assert_eq!(binding.provider.as_deref(), Some("openai"));
    assert_eq!(binding.model.as_deref(), Some("gpt-4o-mini"));
    assert_eq!(
        binding.base_url.as_deref(),
        Some("http://tenant-a.local/v1")
    );
    assert_eq!(binding.api_key_env.as_deref(), Some("TENANT_A_API_KEY"));
}
