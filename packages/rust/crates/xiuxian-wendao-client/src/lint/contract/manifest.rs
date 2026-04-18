#![cfg(test)]

use std::collections::BTreeMap;

use super::assets::{MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID, markdown_lint_diagnostic_manifest};
use super::snapshot::{
    MarkdownLintCliContractSnapshot, MarkdownLintContractDefaultValue,
    MarkdownLintContractParamSnapshot, MarkdownLintDiagnosticContractSnapshot,
    MarkdownLintDiagnosticOutputSnapshot, MarkdownLintRuleContractSnapshot,
};
use serde::Deserialize;

const CONTRACTS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/contracts");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct MarkdownLintDiagnosticManifest {
    pub(super) id: String,
    pub(super) version: u32,
    pub(super) task_types: Vec<String>,
    pub(super) cli: MarkdownLintCliManifest,
    pub(super) output: MarkdownLintDiagnosticManifestOutput,
    pub(super) params: Vec<MarkdownLintContractParamManifest>,
    pub(super) rules: Vec<MarkdownLintRuleContractManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct MarkdownLintCliManifest {
    pub(super) argv: Vec<String>,
    #[serde(default)]
    pub(super) positionals: Vec<String>,
    pub(super) flags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct MarkdownLintDiagnosticManifestOutput {
    pub(super) format: String,
    pub(super) schema_provider: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct MarkdownLintContractParamManifest {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) value_type: String,
    #[serde(default)]
    pub(super) required: bool,
    #[serde(default)]
    pub(super) default: Option<MarkdownLintContractDefaultValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct MarkdownLintRuleContractManifest {
    pub(super) code: String,
    #[serde(default)]
    pub(super) problem: Option<String>,
    #[serde(default)]
    pub(super) problem_strategy: Option<String>,
    #[serde(default)]
    pub(super) detail: Option<String>,
    #[serde(default)]
    pub(super) detail_strategy: Option<String>,
    #[serde(default)]
    pub(super) found: Option<String>,
    #[serde(default)]
    pub(super) found_strategy: Option<String>,
    #[serde(default)]
    pub(super) expected: Option<String>,
    #[serde(default)]
    pub(super) expected_strategy: Option<String>,
    #[serde(default)]
    pub(super) tip: Option<String>,
    #[serde(default)]
    pub(super) tip_strategy: Option<String>,
}

pub(super) fn parse_manifest(contract_id: &str) -> anyhow::Result<MarkdownLintDiagnosticManifest> {
    let raw = markdown_lint_diagnostic_manifest(contract_id).ok_or_else(|| {
        anyhow::anyhow!("missing markdown lint contract manifest for `{contract_id}`")
    })?;
    toml::from_str(raw).map_err(|error| {
        anyhow::anyhow!("failed to parse markdown lint contract manifest `{contract_id}`: {error}")
    })
}

pub(super) fn generate_snapshot_contract_toml(contract_id: &str) -> anyhow::Result<String> {
    let manifest = parse_manifest(contract_id)?;
    validate_manifest(&manifest)?;
    let mut rendered = toml::to_string_pretty(&build_snapshot(&manifest)).map_err(|error| {
        anyhow::anyhow!(
            "failed to serialize markdown lint contract snapshot `{contract_id}`: {error}"
        )
    })?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

pub(super) fn contract_snapshot_path(contract_id: &str) -> String {
    format!("{CONTRACTS_ROOT}/snapshots/{contract_id}/contract.toml")
}

pub(super) fn schema_snapshot_path(contract_id: &str) -> String {
    format!("{CONTRACTS_ROOT}/snapshots/{contract_id}/schema.json")
}

pub(super) fn snapshot_root_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/resources/contracts/snapshots")
}

fn build_snapshot(
    manifest: &MarkdownLintDiagnosticManifest,
) -> MarkdownLintDiagnosticContractSnapshot {
    MarkdownLintDiagnosticContractSnapshot {
        id: manifest.id.clone(),
        version: manifest.version,
        task_types: manifest.task_types.clone(),
        cli: MarkdownLintCliContractSnapshot {
            argv: manifest.cli.argv.clone(),
            positionals: manifest.cli.positionals.clone(),
            flags: manifest.cli.flags.clone(),
        },
        output: MarkdownLintDiagnosticOutputSnapshot {
            format: manifest.output.format.clone(),
            schema: "schema.json".to_string(),
        },
        params: manifest
            .params
            .iter()
            .map(|param| MarkdownLintContractParamSnapshot {
                name: param.name.clone(),
                value_type: param.value_type.clone(),
                required: param.required,
                default: param.default.clone(),
            })
            .collect(),
        rules: manifest
            .rules
            .iter()
            .map(|rule| MarkdownLintRuleContractSnapshot {
                code: rule.code.clone(),
                problem: rule.problem.clone(),
                problem_strategy: rule.problem_strategy.clone(),
                detail: rule.detail.clone(),
                detail_strategy: rule.detail_strategy.clone(),
                found: rule.found.clone(),
                found_strategy: rule.found_strategy.clone(),
                expected: rule.expected.clone(),
                expected_strategy: rule.expected_strategy.clone(),
                tip: rule.tip.clone(),
                tip_strategy: rule.tip_strategy.clone(),
            })
            .collect(),
    }
}

fn validate_manifest(manifest: &MarkdownLintDiagnosticManifest) -> anyhow::Result<()> {
    let expected = expected_contract_shape(manifest.id.as_str())
        .ok_or_else(|| anyhow::anyhow!("unsupported markdown lint contract `{}`", manifest.id))?;
    if manifest.version != 1 {
        anyhow::bail!(
            "markdown lint contract `{}` must stay on version 1, got {}",
            manifest.id,
            manifest.version
        );
    }
    if manifest.task_types != expected.task_types {
        anyhow::bail!(
            "markdown lint contract `{}` task_types drifted",
            manifest.id
        );
    }
    if manifest.cli != expected.cli {
        anyhow::bail!(
            "markdown lint contract `{}` cli surface drifted",
            manifest.id
        );
    }
    if manifest.output != expected.output {
        anyhow::bail!("markdown lint contract `{}` output drifted", manifest.id);
    }
    if manifest.params != expected.params {
        anyhow::bail!("markdown lint contract `{}` params drifted", manifest.id);
    }
    if manifest.rules != expected.rules {
        anyhow::bail!("markdown lint contract `{}` rules drifted", manifest.id);
    }
    Ok(())
}

fn expected_contract_shape(contract_id: &str) -> Option<MarkdownLintDiagnosticManifest> {
    match contract_id {
        MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID => Some(markdown_lint_diagnostics_contract_shape()),
        _ => None,
    }
}

fn markdown_lint_diagnostics_contract_shape() -> MarkdownLintDiagnosticManifest {
    MarkdownLintDiagnosticManifest {
        id: MARKDOWN_LINT_DIAGNOSTICS_CONTRACT_ID.to_string(),
        version: 1,
        task_types: vec!["cli_call".to_string(), "diagnostic_render".to_string()],
        cli: markdown_lint_diagnostics_cli(),
        output: MarkdownLintDiagnosticManifestOutput {
            format: "markdown_lint_report".to_string(),
            schema_provider: "MarkdownLintReport".to_string(),
        },
        params: markdown_lint_diagnostics_params(),
        rules: markdown_lint_diagnostics_rules(),
    }
}

fn markdown_lint_diagnostics_cli() -> MarkdownLintCliManifest {
    MarkdownLintCliManifest {
        argv: vec![
            "wendao".to_string(),
            "lint".to_string(),
            "markdown".to_string(),
        ],
        positionals: vec!["paths".to_string()],
        flags: BTreeMap::from([
            ("output".to_string(), "--output".to_string()),
            ("root".to_string(), "--root".to_string()),
            ("skip_dirs".to_string(), "--skip-dir".to_string()),
        ]),
    }
}

fn markdown_lint_diagnostics_params() -> Vec<MarkdownLintContractParamManifest> {
    vec![
        optional_string_array_param("paths"),
        optional_string_param("root", Some(".")),
        optional_string_param("output", Some("text")),
        optional_string_array_param("skip_dirs"),
    ]
}

fn markdown_lint_diagnostics_rules() -> Vec<MarkdownLintRuleContractManifest> {
    let mut rules = markdown_lint_syntax_rules();
    rules.extend(markdown_lint_obsidian_policy_rules());
    rules.extend(markdown_lint_directory_policy_rules());
    rules
}

fn markdown_lint_syntax_rules() -> Vec<MarkdownLintRuleContractManifest> {
    vec![
        literal_rule(
            "invalid_utf8",
            Some("Markdown file is not valid UTF-8."),
            Some("utf8_error_message"),
            Some("Encode the file as UTF-8 before linting it."),
        ),
        MarkdownLintRuleContractManifest {
            code: "unclosed_frontmatter".to_string(),
            problem: Some("YAML frontmatter opens but never closes.".to_string()),
            problem_strategy: None,
            detail: None,
            detail_strategy: Some("parser_message".to_string()),
            found: Some("---".to_string()),
            found_strategy: None,
            expected: Some(
                "Close the frontmatter with `---` or `...` before the document body begins."
                    .to_string(),
            ),
            expected_strategy: None,
            tip: None,
            tip_strategy: None,
        },
        MarkdownLintRuleContractManifest {
            code: "invalid_frontmatter_yaml".to_string(),
            problem: Some("YAML frontmatter is syntactically invalid.".to_string()),
            problem_strategy: None,
            detail: None,
            detail_strategy: Some("parser_message".to_string()),
            found: None,
            found_strategy: Some("source_line".to_string()),
            expected: Some(
                "Keep valid YAML between the opening and closing frontmatter fences.".to_string(),
            ),
            expected_strategy: None,
            tip: None,
            tip_strategy: None,
        },
        MarkdownLintRuleContractManifest {
            code: "unclosed_fence".to_string(),
            problem: Some("Fenced code block opens but never closes.".to_string()),
            problem_strategy: None,
            detail: None,
            detail_strategy: Some("parser_message".to_string()),
            found: None,
            found_strategy: Some("source_line".to_string()),
            expected: Some(
                "Add a closing fence with the same marker type and at least the same width."
                    .to_string(),
            ),
            expected_strategy: None,
            tip: None,
            tip_strategy: None,
        },
    ]
}

fn markdown_lint_obsidian_policy_rules() -> Vec<MarkdownLintRuleContractManifest> {
    vec![
        MarkdownLintRuleContractManifest {
            code: "bare_obsidian_wikilink".to_string(),
            problem: Some("Obsidian officially allows bare wikilinks, but repository authoring policy requires an explicit display label.".to_string()),
            problem_strategy: None,
            detail: Some("Keep the official Obsidian target, but add a descriptive display label for repository and LLM-facing authoring.".to_string()),
            detail_strategy: None,
            found: None,
            found_strategy: Some("link_literal".to_string()),
            expected: None,
            expected_strategy: Some("rewrite_with_markdown".to_string()),
            tip: None,
            tip_strategy: Some("display_label_tip".to_string()),
        },
        MarkdownLintRuleContractManifest {
            code: "redundant_obsidian_label".to_string(),
            problem: None,
            problem_strategy: Some("redundant_label_problem".to_string()),
            detail: Some("The explicit label should add human-readable namespace meaning instead of echoing the raw path or heading.".to_string()),
            detail_strategy: None,
            found: None,
            found_strategy: Some("link_literal".to_string()),
            expected: None,
            expected_strategy: Some("rewrite_with_markdown".to_string()),
            tip: None,
            tip_strategy: Some("display_label_tip".to_string()),
        },
        MarkdownLintRuleContractManifest {
            code: "mixed_wikilink_markdown_link".to_string(),
            problem: Some("Wikilink brackets and Markdown link parentheses are mixed into one invalid link under Obsidian official syntax.".to_string()),
            problem_strategy: None,
            detail: Some("Choose either official Obsidian wikilink syntax or standard Markdown link syntax.".to_string()),
            detail_strategy: None,
            found: None,
            found_strategy: Some("link_literal".to_string()),
            expected: None,
            expected_strategy: Some("rewrite_with_markdown".to_string()),
            tip: None,
            tip_strategy: Some("display_label_tip".to_string()),
        },
        MarkdownLintRuleContractManifest {
            code: "non_canonical_obsidian_alias_order".to_string(),
            problem: Some("The right-hand wikilink segment looks like a repository target path or address, so target and display label appear reversed.".to_string()),
            problem_strategy: None,
            detail: Some("This is legal Obsidian wikilink syntax, but repository authoring policy only flags reversed alias order when the right side looks like a path, heading address, block address, or Markdown note target.".to_string()),
            detail_strategy: None,
            found: None,
            found_strategy: Some("link_literal".to_string()),
            expected: None,
            expected_strategy: Some("rewrite_wikilink_only".to_string()),
            tip: None,
            tip_strategy: Some("display_label_tip".to_string()),
        },
    ]
}

fn markdown_lint_directory_policy_rules() -> Vec<MarkdownLintRuleContractManifest> {
    vec![
        MarkdownLintRuleContractManifest {
            code: "directory_link_style_mismatch".to_string(),
            problem: None,
            problem_strategy: Some("dynamic_problem_text".to_string()),
            detail: None,
            detail_strategy: Some("dynamic_detail_text".to_string()),
            found: None,
            found_strategy: Some("dynamic_found_text".to_string()),
            expected: None,
            expected_strategy: Some("dynamic_expected_text".to_string()),
            tip: None,
            tip_strategy: Some("dynamic_tip_text".to_string()),
        },
        MarkdownLintRuleContractManifest {
            code: "directory_link_style_ambiguous".to_string(),
            problem: None,
            problem_strategy: Some("dynamic_problem_text".to_string()),
            detail: None,
            detail_strategy: Some("dynamic_detail_text".to_string()),
            found: None,
            found_strategy: Some("dynamic_found_text".to_string()),
            expected: None,
            expected_strategy: Some("dynamic_expected_text".to_string()),
            tip: None,
            tip_strategy: Some("dynamic_tip_text".to_string()),
        },
    ]
}

fn literal_rule(
    code: &str,
    problem: Option<&str>,
    detail_strategy: Option<&str>,
    expected: Option<&str>,
) -> MarkdownLintRuleContractManifest {
    MarkdownLintRuleContractManifest {
        code: code.to_string(),
        problem: problem.map(ToOwned::to_owned),
        problem_strategy: None,
        detail: None,
        detail_strategy: detail_strategy.map(ToOwned::to_owned),
        found: None,
        found_strategy: None,
        expected: expected.map(ToOwned::to_owned),
        expected_strategy: None,
        tip: None,
        tip_strategy: None,
    }
}

fn optional_string_param(name: &str, default: Option<&str>) -> MarkdownLintContractParamManifest {
    MarkdownLintContractParamManifest {
        name: name.to_string(),
        value_type: "string".to_string(),
        required: false,
        default: default.map(|value| MarkdownLintContractDefaultValue::String(value.to_string())),
    }
}

fn optional_string_array_param(name: &str) -> MarkdownLintContractParamManifest {
    MarkdownLintContractParamManifest {
        name: name.to_string(),
        value_type: "string_array".to_string(),
        required: false,
        default: None,
    }
}
