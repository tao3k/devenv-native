use anyhow::Result;
use tempfile::TempDir;

use super::{run_semantic_lint, run_semantic_lint_with_args};

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
        projection.contains("projection_revision: test.v1"),
        "projection revision should remain unchanged: {projection}"
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
