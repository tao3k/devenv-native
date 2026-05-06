use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

#[path = "semantic/basic.rs"]
mod basic;
#[path = "semantic/lifecycle.rs"]
mod lifecycle;
#[path = "semantic/projection.rs"]
mod projection;
#[path = "semantic/read_model.rs"]
mod read_model;
#[path = "semantic/refresh.rs"]
mod refresh;

fn run_semantic_lint(temp: &TempDir, scope: Option<&str>) -> Result<(Option<i32>, String)> {
    super::run_semantic_lint(temp, scope)
}

fn run_semantic_lint_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    super::run_semantic_lint_with_args(temp, scope, args)
}

fn run_semantic_refresh_projections(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    super::run_semantic_refresh_projections(temp, scope)
}

fn run_semantic_refresh_projections_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    super::run_semantic_refresh_projections_with_args(temp, scope, args)
}

fn run_semantic_refresh_projections_with_args_and_stderr(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    super::run_semantic_refresh_projections_with_args_and_stderr(temp, scope, args)
}

fn run_semantic_describe_read_model(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    super::run_semantic_describe_read_model(temp, scope)
}

fn run_semantic_snapshot_read_model(
    temp: &TempDir,
    scope: Option<&str>,
) -> Result<(Option<i32>, String)> {
    super::run_semantic_snapshot_read_model(temp, scope)
}

fn run_semantic_check_read_model_snapshot_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    super::run_semantic_check_read_model_snapshot_with_args(temp, scope, args)
}

fn run_semantic_plan_read_model_materialization_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    super::run_semantic_plan_read_model_materialization_with_args(temp, scope, args)
}

fn run_semantic_preflight_read_model_materialization_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    super::run_semantic_preflight_read_model_materialization_with_args(temp, scope, args)
}

fn run_semantic_query_read_model_with_args(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String)> {
    super::run_semantic_query_read_model_with_args(temp, scope, args)
}

fn run_semantic_query_read_model_with_args_and_stderr(
    temp: &TempDir,
    scope: Option<&str>,
    args: &[&str],
) -> Result<(Option<i32>, String, String)> {
    super::run_semantic_query_read_model_with_args_and_stderr(temp, scope, args)
}

fn read_snapshot_revision(stdout: &str) -> Result<String> {
    let Some(line) = stdout
        .lines()
        .find(|line| line.starts_with("Semantic read-model snapshot: blake3:"))
    else {
        anyhow::bail!("snapshot output did not contain an aggregate revision: {stdout}");
    };
    let Some(revision) = line
        .strip_prefix("Semantic read-model snapshot: ")
        .and_then(|rest| rest.strip_suffix(" from semantic."))
    else {
        anyhow::bail!("snapshot output revision line had unexpected shape: {line}");
    };
    Ok(revision.to_string())
}

fn initialize_git_fixture(temp: &TempDir) -> Result<()> {
    run_git(temp, &["init"])?;
    run_git(
        temp,
        &["config", "user.email", "semantic-test@example.invalid"],
    )?;
    run_git(temp, &["config", "user.name", "Semantic Test"])?;
    run_git(temp, &["add", "."])?;
    run_git(temp, &["commit", "-m", "fixture"])?;
    Ok(())
}

fn run_git(temp: &TempDir, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(args)
        .output()?;
    assert!(
        output.status.success(),
        "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn write_semantic_fixture(
    temp: &TempDir,
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
) -> Result<()> {
    write_semantic_fixture_with_relation(temp, id, kind, title, status, "  []\n")
}

fn write_semantic_fixture_with_relation(
    temp: &TempDir,
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    relations: &str,
) -> Result<()> {
    let object_dir = temp.path().join("semantic/objects").join(kind);
    std::fs::create_dir_all(&object_dir)?;
    std::fs::create_dir_all(temp.path().join("semantic/projections"))?;
    std::fs::write(
        object_dir.join("fixture.md"),
        format!(
            concat!(
                "---\n",
                "id: {id}\n",
                "kind: {kind}\n",
                "title: {title}\n",
                "status: {status}\n",
                "confidence:\n",
                "  score: 1.0\n",
                "  source: human_signed\n",
                "owners:\n",
                "  - scope: tests\n",
                "    role: fixture\n",
                "provenance:\n",
                "  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md\n",
                "  recorded_by: codex\n",
                "  recorded_at: \"2026-05-05\"\n",
                "verification:\n",
                "  required:\n",
                "    - direnv exec . wendao-client lint semantic\n",
                "relations:\n",
                "{relations}",
                "---\n",
                "# {title}\n",
            ),
            id = id,
            kind = kind,
            title = title,
            status = status,
            relations = relations,
        ),
    )?;
    std::fs::write(
        temp.path().join("semantic/projections/llm-compression.md"),
        format!(
            concat!(
                "---\n",
                "type: semantic_projection\n",
                "projection: llm_compression\n",
                "source_objects:\n",
                "  - {id}\n",
                "source_revision: stale-fixture\n",
                "projection_revision: test.v1\n",
                "staleness: stale\n",
                "status: active\n",
                "---\n",
                "# Projection\n",
            ),
            id = id,
        ),
    )?;
    Ok(())
}

fn write_semantic_lifecycle_fixture(temp: &TempDir) -> Result<()> {
    let task_dir = temp.path().join("semantic/objects/task");
    let invariant_dir = temp.path().join("semantic/objects/invariant");
    std::fs::create_dir_all(&task_dir)?;
    std::fs::create_dir_all(&invariant_dir)?;
    std::fs::create_dir_all(temp.path().join("semantic/projections"))?;
    std::fs::create_dir_all(temp.path().join("semantic/change-intents"))?;
    std::fs::write(
        task_dir.join("accepted.md"),
        semantic_object_fixture("task.accepted", "task", "Accepted Task", "active", "  []\n"),
    )?;
    std::fs::write(
        invariant_dir.join("fixture.md"),
        semantic_object_fixture(
            "invariant.fixture",
            "invariant",
            "Invariant Fixture",
            "active",
            "  []\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("semantic/projections/llm-compression.md"),
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "  - task.accepted\n",
            "  - invariant.fixture\n",
            "source_revision: stale-fixture\n",
            "projection_revision: test.v1\n",
            "staleness: stale\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("semantic/change-intents/lifecycle.md"),
        concat!(
            "---\n",
            "type: semantic_change_intent\n",
            "id: change.fixture.lifecycle\n",
            "title: Lifecycle Fixture\n",
            "status: active\n",
            "touched_objects:\n",
            "  - task.accepted\n",
            "changed_relations: []\n",
            "status_transitions:\n",
            "  - object_id: task.accepted\n",
            "    from: candidate\n",
            "    to: active\n",
            "promotion_targets:\n",
            "  - task.accepted\n",
            "demotion_targets: []\n",
            "affected_invariants:\n",
            "  - invariant.fixture\n",
            "required_validations:\n",
            "  - direnv exec . wendao-client lint semantic\n",
            "projections_to_refresh:\n",
            "  - llm_compression\n",
            "candidate_suggestions: []\n",
            "---\n",
            "# Lifecycle Fixture\n",
        ),
    )?;
    Ok(())
}

fn write_pending_semantic_lifecycle_fixture(temp: &TempDir) -> Result<()> {
    let task_dir = temp.path().join("semantic/objects/task");
    let invariant_dir = temp.path().join("semantic/objects/invariant");
    std::fs::create_dir_all(&task_dir)?;
    std::fs::create_dir_all(&invariant_dir)?;
    std::fs::create_dir_all(temp.path().join("semantic/projections"))?;
    std::fs::create_dir_all(temp.path().join("semantic/change-intents"))?;
    std::fs::write(
        task_dir.join("accepted.md"),
        semantic_object_fixture_with_confidence(
            "task.accepted",
            "task",
            "Accepted Task",
            "candidate",
            "llm_suggested",
            "  []\n",
        ),
    )?;
    std::fs::write(
        invariant_dir.join("fixture.md"),
        semantic_object_fixture(
            "invariant.fixture",
            "invariant",
            "Invariant Fixture",
            "active",
            "  []\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("semantic/projections/llm-compression.md"),
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "  - task.accepted\n",
            "  - invariant.fixture\n",
            "source_revision: stale-fixture\n",
            "projection_revision: test.v1\n",
            "staleness: stale\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
    )?;
    std::fs::write(
        temp.path().join("semantic/change-intents/lifecycle.md"),
        concat!(
            "---\n",
            "type: semantic_change_intent\n",
            "id: change.fixture.lifecycle\n",
            "title: Lifecycle Fixture\n",
            "status: active\n",
            "touched_objects:\n",
            "  - task.accepted\n",
            "changed_relations: []\n",
            "status_transitions:\n",
            "  - object_id: task.accepted\n",
            "    from: candidate\n",
            "    to: active\n",
            "promotion_targets:\n",
            "  - task.accepted\n",
            "demotion_targets: []\n",
            "affected_invariants:\n",
            "  - invariant.fixture\n",
            "required_validations:\n",
            "  - direnv exec . wendao-client lint semantic\n",
            "projections_to_refresh:\n",
            "  - llm_compression\n",
            "candidate_suggestions:\n",
            "  - task.accepted\n",
            "---\n",
            "# Lifecycle Fixture\n",
        ),
    )?;
    Ok(())
}

fn semantic_object_fixture(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    relations: &str,
) -> String {
    semantic_object_fixture_with_confidence(id, kind, title, status, "human_signed", relations)
}

fn semantic_object_fixture_with_confidence(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    confidence_source: &str,
    relations: &str,
) -> String {
    format!(
        concat!(
            "---\n",
            "id: {id}\n",
            "kind: {kind}\n",
            "title: {title}\n",
            "status: {status}\n",
            "confidence:\n",
            "  score: 1.0\n",
            "  source: {confidence_source}\n",
            "owners:\n",
            "  - scope: tests\n",
            "    role: fixture\n",
            "provenance:\n",
            "  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md\n",
            "  recorded_by: codex\n",
            "  recorded_at: \"2026-05-05\"\n",
            "verification:\n",
            "  required:\n",
            "    - direnv exec . wendao-client lint semantic\n",
            "relations:\n",
            "{relations}",
            "---\n",
            "# {title}\n",
        ),
        id = id,
        kind = kind,
        title = title,
        status = status,
        confidence_source = confidence_source,
        relations = relations,
    )
}
