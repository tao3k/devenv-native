use super::resolve_qianji_workflow_llm_task_config_with_env;
use crate::runtime_config::QianjiRuntimeEnv;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn workflow_llm_task_config_loads_system_profile() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for workflow config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    write_file(
        &project_root.join(
            "packages/rust/crates/xiuxian-qianji/resources/config/workflows/bpmn-host-work-llm.toml",
        ),
        r#"
schema = "qianji.workflow.llm_task.v1"

[llm]
provider = "openrouter"
model = "deepseek/test"
wire_api = "chat_completions"

[task]
activity_type = "llm.plan"
task_queue = "llm.openrouter"
max_tokens = 1024
timeout_ms = 45000
"#,
    );

    let config = must_ok(
        resolve_qianji_workflow_llm_task_config_with_env(
            "bpmn-host-work-llm",
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "workflow llm task config should resolve",
    );

    assert_eq!(config.llm.provider.as_deref(), Some("openrouter"));
    assert_eq!(config.llm.model.as_deref(), Some("deepseek/test"));
    assert_eq!(config.task.activity_type.as_deref(), Some("llm.plan"));
    assert_eq!(config.task.task_queue.as_deref(), Some("llm.openrouter"));
    assert_eq!(config.task.max_tokens, Some(1024));
    assert_eq!(config.task.timeout_ms, Some(45000));
}

#[test]
fn workflow_llm_task_config_user_overlay_overrides_task_fields() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for workflow config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    let system_profile = project_root.join(
        "packages/rust/crates/xiuxian-qianji/resources/config/workflows/bpmn-host-work-llm.toml",
    );
    let user_profile =
        config_home.join("xiuxian-artisan-workshop/workflows/bpmn-host-work-llm.toml");
    write_file(
        &system_profile,
        r#"
[llm]
model = "system-model"

[task]
activity_type = "llm.plan"
task_queue = "llm.openrouter"
max_tokens = 1024
"#,
    );
    write_file(
        &user_profile,
        r#"
[task]
task_queue = "llm.local"
max_tokens = 2048

[task.retry]
max_attempts = 3
non_retryable_error_codes = ["SchemaInvalid"]
"#,
    );

    let config = must_ok(
        resolve_qianji_workflow_llm_task_config_with_env(
            "bpmn-host-work-llm",
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "workflow llm task config should resolve",
    );

    assert_eq!(config.llm.model.as_deref(), Some("system-model"));
    assert_eq!(config.task.activity_type.as_deref(), Some("llm.plan"));
    assert_eq!(config.task.task_queue.as_deref(), Some("llm.local"));
    assert_eq!(config.task.max_tokens, Some(2048));
    let Some(retry) = config.task.retry else {
        panic!("retry overlay should apply");
    };
    assert_eq!(retry.max_attempts, Some(3));
    assert_eq!(retry.non_retryable_error_codes, ["SchemaInvalid"]);
}

#[test]
fn workflow_llm_task_config_rejects_path_traversal_profile() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for workflow config test: {err}"));

    let error = must_err(
        resolve_qianji_workflow_llm_task_config_with_env(
            "../qianji",
            &QianjiRuntimeEnv {
                prj_root: Some(tmp.path().join("project")),
                prj_config_home: Some(tmp.path().join("project/.config")),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "path traversal profile should fail",
    );

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!(
                "failed to create parent directory '{}': {err}",
                parent.display()
            )
        });
    }
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write file '{}': {err}", path.display()));
}

fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_err<T, E>(result: Result<T, E>, context: &str) -> E
where
    E: std::fmt::Display,
{
    match result {
        Ok(_) => panic!("{context}: expected error"),
        Err(error) => error,
    }
}
