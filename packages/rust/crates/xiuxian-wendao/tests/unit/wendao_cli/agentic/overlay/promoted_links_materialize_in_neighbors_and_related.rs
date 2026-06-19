use crate::test_wendao_cli::agentic::overlay::support::{
    assert_promoted_overlay_applied, assert_verbose_overlay, run_wendao_json, run_wendao_ok,
    write_agentic_config,
};
use crate::test_wendao_cli::support::{clear_valkey_prefix, unique_agentic_prefix, write_file};
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn seed_overlay_docs(tmp: &TempDir) -> TestResult {
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nbeta\n")?;
    Ok(())
}

fn promote_logged_suggestion(config_path: &Path) -> TestResult {
    let log_payload = run_wendao_json(
        None,
        config_path,
        &[
            "agentic",
            "log",
            "docs/a.md",
            "docs/b.md",
            "related_to",
            "--confidence",
            "0.91",
            "--evidence",
            "promoted-link-test",
            "--agent-id",
            "wendao-agentic-architect",
        ],
        "wendao agentic log failed",
    )?;
    let suggestion_id = log_payload
        .get("suggestion_id")
        .and_then(Value::as_str)
        .ok_or("missing suggestion_id")?;
    run_wendao_ok(
        None,
        config_path,
        &[
            "agentic",
            "decide",
            suggestion_id,
            "--target-state",
            "promoted",
            "--decided-by",
            "omega-gate",
            "--reason",
            "promotion for retrieval overlay",
        ],
        "wendao agentic decide failed",
    )
}

fn assert_neighbors_include_promoted(root: &Path, config_path: &Path) -> TestResult {
    let neighbors_payload = run_wendao_json(
        Some(root),
        config_path,
        &[
            "neighbors",
            "a",
            "--direction",
            "outgoing",
            "--hops",
            "1",
            "--limit",
            "10",
        ],
        "wendao neighbors failed",
    )?;
    let neighbors = neighbors_payload
        .as_array()
        .ok_or("neighbors payload must be array")?;
    assert!(neighbors.iter().any(|row| {
        row.get("stem").and_then(Value::as_str) == Some("b")
            && row.get("path").and_then(Value::as_str) == Some("docs/b.md")
    }));

    let neighbors_verbose_payload = run_wendao_json(
        Some(root),
        config_path,
        &[
            "neighbors",
            "a",
            "--direction",
            "outgoing",
            "--hops",
            "1",
            "--limit",
            "10",
            "--verbose",
        ],
        "wendao neighbors --verbose failed",
    )?;
    assert_verbose_overlay(&neighbors_verbose_payload)
}

fn assert_related_includes_promoted(root: &Path, config_path: &Path) -> TestResult {
    let related_payload = run_wendao_json(
        Some(root),
        config_path,
        &["related", "a", "--max-distance", "2", "--limit", "10"],
        "wendao related failed",
    )?;
    let related_rows = related_payload
        .as_array()
        .ok_or("related payload must be array")?;
    assert!(
        related_rows
            .iter()
            .any(|row| row.get("stem").and_then(Value::as_str) == Some("b")),
        "expected promoted edge to affect related traversal: payload={related_payload}"
    );
    Ok(())
}

fn assert_search_overlay(root: &Path, config_path: &Path) -> TestResult {
    let search_payload = run_wendao_json(
        Some(root),
        config_path,
        &["search", "alpha", "--limit", "5"],
        "wendao search failed",
    )?;
    assert_promoted_overlay_applied(&search_payload)?;
    let overlay = search_payload
        .get("promoted_overlay")
        .ok_or("missing promoted_overlay telemetry")?;
    assert!(
        overlay
            .get("promoted_rows")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 1
    );
    assert!(
        overlay
            .get("added_edges")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 1
    );
    Ok(())
}

#[test]
fn test_wendao_promoted_links_materialize_in_neighbors_and_related() -> TestResult {
    let tmp = TempDir::new()?;
    seed_overlay_docs(&tmp)?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    write_agentic_config(&config_path, &prefix)?;
    promote_logged_suggestion(&config_path)?;
    assert_neighbors_include_promoted(tmp.path(), &config_path)?;
    assert_related_includes_promoted(tmp.path(), &config_path)?;
    assert_search_overlay(tmp.path(), &config_path)?;

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
