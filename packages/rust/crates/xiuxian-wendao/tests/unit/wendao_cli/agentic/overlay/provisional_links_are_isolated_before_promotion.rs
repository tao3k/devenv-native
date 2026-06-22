use crate::test_wendao_cli::agentic::overlay::support::{run_wendao_json, run_wendao_ok};
use crate::test_wendao_cli::support::{clear_valkey_prefix, unique_agentic_prefix, write_file};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn seed_overlay_docs(tmp: &TempDir) -> TestResult {
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nbeta\n")?;
    Ok(())
}

fn write_provisional_config(config_path: &Path, prefix: &str) -> TestResult {
    fs::write(
        config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n    search:\n      include_provisional_default: false\n      provisional_limit: 10\n"
        ),
    )?;
    Ok(())
}

fn log_provisional_suggestion(config_path: &Path) -> TestResult {
    run_wendao_ok(
        None,
        config_path,
        &[
            "agentic",
            "log",
            "docs/a.md",
            "docs/b.md",
            "related_to",
            "--confidence",
            "0.92",
            "--evidence",
            "provisional-only",
            "--agent-id",
            "wendao-agentic-architect",
        ],
        "wendao agentic log failed",
    )
}

fn assert_overlay_not_applied(payload: &Value, context: &str) {
    assert_eq!(
        payload
            .get("promoted_overlay")
            .and_then(|row| row.get("applied"))
            .and_then(Value::as_bool),
        Some(false),
        "{context}: payload={payload}"
    );
}

fn assert_verbose_command_excludes_stem_b(
    root: &Path,
    config_path: &Path,
    args: &[&str],
    results_context: &str,
    command_context: &str,
) -> TestResult {
    let payload = run_wendao_json(Some(root), config_path, args, command_context)?;
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or(results_context)?;
    assert!(
        !rows
            .iter()
            .any(|row| row.get("stem").and_then(Value::as_str) == Some("b")),
        "{results_context}: payload={payload}"
    );
    assert_overlay_not_applied(&payload, results_context);
    Ok(())
}

fn assert_default_search_excludes_stem_b(root: &Path, config_path: &Path) -> TestResult {
    let payload = run_wendao_json(
        Some(root),
        config_path,
        &["search", "alpha", "--limit", "10"],
        "wendao search failed",
    )?;
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("search payload missing results")?;
    assert!(
        !rows
            .iter()
            .any(|row| row.get("stem").and_then(Value::as_str) == Some("b")),
        "provisional link leaked into default search results before promotion: payload={payload}"
    );
    assert_overlay_not_applied(&payload, "search payload must not apply promoted overlay");
    Ok(())
}

#[test]
fn test_wendao_provisional_links_are_isolated_before_promotion() -> TestResult {
    let tmp = TempDir::new()?;
    seed_overlay_docs(&tmp)?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    write_provisional_config(&config_path, &prefix)?;
    log_provisional_suggestion(&config_path)?;
    assert_verbose_command_excludes_stem_b(
        tmp.path(),
        &config_path,
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
        "neighbors verbose payload missing results",
        "wendao neighbors --verbose failed",
    )?;
    assert_verbose_command_excludes_stem_b(
        tmp.path(),
        &config_path,
        &[
            "related",
            "a",
            "--max-distance",
            "2",
            "--limit",
            "10",
            "--verbose",
        ],
        "related verbose payload missing results",
        "wendao related --verbose failed",
    )?;
    assert_default_search_excludes_stem_b(tmp.path(), &config_path)?;

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
