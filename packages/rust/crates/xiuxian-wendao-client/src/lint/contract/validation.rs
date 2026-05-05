use super::assets::MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID;
use super::snapshot::MarkdownLintDiagnosticContractSnapshot;
use crate::lint::diagnostic::{code_string, markdown_lint_issue_codes, markdown_lint_rule_keys};
use anyhow::{Result, bail};

pub(super) fn validate_snapshot(snapshot: &MarkdownLintDiagnosticContractSnapshot) -> Result<()> {
    if snapshot.id != MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID.as_str() {
        bail!(
            "markdown lint diagnostic snapshot id drifted: expected `{}`, got `{}`",
            MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID,
            snapshot.id
        );
    }
    if snapshot.version != 1 {
        bail!(
            "markdown lint diagnostic snapshot version drifted: expected `1`, got `{}`",
            snapshot.version
        );
    }
    if snapshot.task_types != vec!["cli_call".to_string(), "diagnostic_render".to_string()] {
        bail!("markdown lint diagnostic snapshot task_types drifted");
    }
    if snapshot.cli.argv
        != vec![
            "wendao".to_string(),
            "lint".to_string(),
            "markdown".to_string(),
        ]
    {
        bail!("markdown lint diagnostic snapshot cli argv drifted");
    }
    if snapshot.cli.positionals != vec!["paths".to_string()] {
        bail!("markdown lint diagnostic snapshot cli positionals drifted");
    }
    if snapshot
        .cli
        .flags
        .get("root")
        .is_none_or(|flag| flag != "--root")
        || snapshot
            .cli
            .flags
            .get("output")
            .is_none_or(|flag| flag != "--output")
        || snapshot
            .cli
            .flags
            .get("skip_dirs")
            .is_none_or(|flag| flag != "--skip-dir")
    {
        bail!("markdown lint diagnostic snapshot cli flags drifted");
    }
    if snapshot.output.format != "markdown_lint_report" {
        bail!("markdown lint diagnostic snapshot output format drifted");
    }
    if snapshot.output.schema != "schema.json" {
        bail!("markdown lint diagnostic snapshot schema filename drifted");
    }
    if snapshot.params.len() != 4 {
        bail!("markdown lint diagnostic snapshot params drifted");
    }
    Ok(())
}

pub(super) fn validate_required_field(
    key: &str,
    field: &str,
    has_literal: bool,
    has_strategy: bool,
) -> Result<()> {
    match (has_literal, has_strategy) {
        (true, false) | (false, true) => Ok(()),
        (false, false) => bail!(
            "markdown lint diagnostic contract rule `{key}` must set `{field}` or `{field}_strategy`"
        ),
        (true, true) => bail!(
            "markdown lint diagnostic contract rule `{key}` cannot set both `{field}` and `{field}_strategy`"
        ),
    }
}

pub(super) fn validate_optional_field(
    key: &str,
    field: &str,
    has_literal: bool,
    has_strategy: bool,
) -> Result<()> {
    if has_literal && has_strategy {
        bail!(
            "markdown lint diagnostic contract rule `{key}` cannot set both `{field}` and `{field}_strategy`"
        );
    }
    Ok(())
}

pub(super) fn normalize_rule_key(raw_key: &str) -> Option<&str> {
    if matches!(
        raw_key,
        "invalid_utf8"
            | "missing_local_target"
            | "missing_local_fragment"
            | "local_target_outside_root"
            | "local_target_transient_dir"
            | "directory_link_style_mismatch"
            | "directory_link_style_ambiguous"
    ) {
        return Some(raw_key);
    }
    markdown_lint_issue_codes()
        .into_iter()
        .find(|candidate| code_string(*candidate) == raw_key)
        .map(code_string)
}

pub(super) fn known_rule_keys() -> Vec<&'static str> {
    let mut keys = vec!["invalid_utf8"];
    keys.extend(markdown_lint_rule_keys());
    keys
}
