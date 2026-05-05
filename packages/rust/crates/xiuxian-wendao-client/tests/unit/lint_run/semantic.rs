use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

use super::{
    run_semantic_lint, run_semantic_lint_with_args, run_semantic_refresh_projections,
    run_semantic_refresh_projections_with_args,
    run_semantic_refresh_projections_with_args_and_stderr,
};

#[test]
fn semantic_lint_accepts_valid_semantic_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint(&temp, None)?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains(
            "Semantic lint passed: checked 1 root(s), 1 object(s), 1 projection(s), 0 change intent(s), 0 issue(s)."
        ),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_reports_unresolved_relations() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture_with_relation(
        &temp,
        "task.fixture",
        "task",
        "Task Fixture",
        "active",
        "  - kind: depends_on\n    target: component.missing\n",
    )?;

    let (status, stdout) = run_semantic_lint(&temp, None)?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("component.missing"),
        "unresolved target should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_sql_guard_reports_stale_projection() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--semantic-sql-guard"])?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("SQL guard semantic_sql.projection_freshness review_required"),
        "SQL guard status should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("1 failing row(s)"),
        "SQL guard failing row count should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_renders_read_model_summary() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--read-model-summary"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Read-model summary projected"),
        "read-model status should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_objects 1 row(s)"),
        "object row count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("semantic_projection_state 1 row(s)"),
        "projection-state row count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("repo-native semantic artifacts remain authoritative"),
        "authority boundary should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_refreshes_projection_source_revision() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--refresh-projections"])?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "refresh count should be rendered: {stdout}"
    );
    let projection =
        std::fs::read_to_string(temp.path().join("semantic/projections/llm-compression.md"))?;
    assert!(
        !projection.contains("source_revision: stale-fixture"),
        "stale source revision should be replaced: {projection}"
    );
    assert!(
        projection.contains("staleness: fresh"),
        "staleness should be marked fresh: {projection}"
    );
    assert!(
        projection.contains("source_objects:\n  - decision.fixture"),
        "projection refresh should preserve block sequence indentation: {projection}"
    );
    assert!(
        projection.contains("source_revision: \"blake3:"),
        "projection refresh should keep source revision quoted: {projection}"
    );
    assert!(
        projection.contains("projection_revision: test.v1"),
        "projection revision should remain unchanged: {projection}"
    );

    let (status, stdout) = run_semantic_lint(&temp, None)?;
    assert_eq!(status, Some(0), "{stdout}");
    Ok(())
}

#[test]
fn semantic_lint_reports_lifecycle_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--lifecycle-plan"])?;

    assert_eq!(status, Some(0));
    assert!(
        stdout.contains(
            "Lifecycle plan 1 promotion(s), 0 demotion(s), 0 other transition(s), 0 pending apply target(s), 1 already-applied writeback target(s), 0 blocked target(s)."
        ),
        "lifecycle plan summary should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "change.fixture.lifecycle: task.accepted candidate -> active (promotion, already_applied)"
        ),
        "lifecycle plan entry should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_requires_fresh_projection_refresh_targets() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--require-fresh-projections"])?;

    assert_eq!(status, Some(1));
    assert!(
        stdout.contains("1 projection policy issue(s)"),
        "projection policy issue count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets review_required"
        ),
        "projection policy failure should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression (stale, stale)"),
        "stale projection entry should be rendered: {stdout}"
    );

    let (status, stdout) = run_semantic_lint_with_args(
        &temp,
        None,
        &["--refresh-projections", "--require-fresh-projections"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "refresh count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets passed (0 failing projection(s))"
        ),
        "projection policy pass should be rendered: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_lint_renders_projection_refresh_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--projection-refresh-plan"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan refresh_required"),
        "projection refresh plan should be rendered: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression -> refresh_source_revision (stale, stale)"),
        "projection refresh entry should be rendered: {stdout}"
    );

    let (status, stdout) = run_semantic_lint_with_args(
        &temp,
        None,
        &["--refresh-projections", "--projection-refresh-plan"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan up_to_date (0 refreshable projection(s))"),
        "refreshed projection should make plan empty: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_command_runs_one_worker_pass() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_refresh_projections(&temp, None)?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "worker should refresh stale projection metadata: {stdout}"
    );
    assert!(
        stdout.contains("Projection refresh plan up_to_date"),
        "worker should report an empty post-refresh plan: {stdout}"
    );
    assert!(
        stdout.contains(
            "Projection freshness policy semantic_projection.required_refresh_targets passed"
        ),
        "worker should enforce post-refresh projection freshness: {stdout}"
    );

    let projection =
        std::fs::read_to_string(temp.path().join("semantic/projections/llm-compression.md"))?;
    assert!(
        projection.contains("staleness: fresh"),
        "worker should mark projection metadata fresh: {projection}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_command_runs_bounded_repeated_worker_passes() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;

    let (status, stdout) = run_semantic_refresh_projections_with_args(
        &temp,
        None,
        &["--interval-secs", "0", "--max-runs", "2"],
    )?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "first worker pass should refresh stale projection metadata: {stdout}"
    );
    assert_eq!(
        stdout.matches("Projection refresh plan up_to_date").count(),
        2,
        "bounded runner should render a post-refresh plan for each pass: {stdout}"
    );
    assert_eq!(
        stdout
            .matches(
                "Projection freshness policy semantic_projection.required_refresh_targets passed"
            )
            .count(),
        2,
        "bounded runner should enforce freshness for each pass: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_clean_worktree_guard_accepts_clean_git_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    initialize_git_fixture(&temp)?;

    let (status, stdout, stderr) = run_semantic_refresh_projections_with_args_and_stderr(
        &temp,
        None,
        &["--require-clean-worktree"],
    )?;

    assert_eq!(status, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stdout.contains("Refreshed 1 semantic projection source revision(s)."),
        "clean root should allow supervised refresh: {stdout}"
    );
    Ok(())
}

#[test]
fn semantic_refresh_projections_clean_worktree_guard_rejects_dirty_git_root() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    initialize_git_fixture(&temp)?;
    std::fs::write(temp.path().join("dirty.md"), "# Dirty\n")?;

    let (status, stdout, stderr) = run_semantic_refresh_projections_with_args_and_stderr(
        &temp,
        None,
        &["--require-clean-worktree"],
    )?;

    assert_eq!(status, Some(1), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        stderr.contains("requires a clean git worktree"),
        "dirty root should be rejected before refresh: {stderr}"
    );
    assert!(
        stderr.contains("dirty.md"),
        "dirty path should be rendered for supervisor triage: {stderr}"
    );
    Ok(())
}

#[test]
fn semantic_lint_renders_projection_refresh_plan_for_fresh_revision_mismatch() -> Result<()> {
    let temp = TempDir::new()?;
    write_semantic_fixture(
        &temp,
        "decision.fixture",
        "decision",
        "Decision Fixture",
        "active",
    )?;
    let projection_path = temp.path().join("semantic/projections/llm-compression.md");
    let projection = std::fs::read_to_string(&projection_path)?;
    std::fs::write(
        &projection_path,
        projection.replace("staleness: stale", "staleness: fresh"),
    )?;

    let (status, stdout) =
        run_semantic_lint_with_args(&temp, None, &["--projection-refresh-plan"])?;

    assert_eq!(status, Some(1), "{stdout}");
    assert!(
        stdout.contains("Projection refresh plan refresh_required"),
        "projection refresh plan should render even for refreshable validation issues: {stdout}"
    );
    assert!(
        stdout.contains("llm_compression -> refresh_source_revision"),
        "projection refresh entry should be rendered: {stdout}"
    );
    Ok(())
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

#[test]
fn semantic_lint_applies_pending_lifecycle_plan() -> Result<()> {
    let temp = TempDir::new()?;
    write_pending_semantic_lifecycle_fixture(&temp)?;

    let (status, stdout) = run_semantic_lint_with_args(&temp, None, &["--apply-lifecycle-plan"])?;

    assert_eq!(status, Some(0), "{stdout}");
    assert!(
        stdout.contains("Applied 1 semantic lifecycle writeback(s)."),
        "apply count should be rendered: {stdout}"
    );
    assert!(
        stdout.contains(
            "Lifecycle plan 1 promotion(s), 0 demotion(s), 0 other transition(s), 0 pending apply target(s), 1 already-applied writeback target(s), 0 blocked target(s)."
        ),
        "post-apply lifecycle plan should be rendered: {stdout}"
    );

    let object = std::fs::read_to_string(temp.path().join("semantic/objects/task/accepted.md"))?;
    assert!(
        object.contains("status: active"),
        "object status should be promoted: {object}"
    );
    assert!(
        object.contains("source: human_signed"),
        "promotion should update confidence source: {object}"
    );
    assert!(
        !object.contains("source: llm_suggested"),
        "promoted object must not keep llm_suggested confidence: {object}"
    );
    let intent = std::fs::read_to_string(temp.path().join("semantic/change-intents/lifecycle.md"))?;
    assert!(
        intent.contains("candidate_suggestions: []"),
        "promoted object should be removed from candidate suggestions: {intent}"
    );

    let (status, stdout) = run_semantic_lint(&temp, None)?;
    assert_eq!(status, Some(0), "{stdout}");
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
