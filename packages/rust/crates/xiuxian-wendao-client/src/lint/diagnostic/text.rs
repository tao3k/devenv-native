use super::link::{LinkIssueContext, TargetMetadata};
use xiuxian_wendao_parsers::{MarkdownLintKind, MarkdownSyntaxLintCode};

const SYNTHETIC_RULE_KEYS: &[&str] = &[
    "directory_link_style_mismatch",
    "directory_link_style_ambiguous",
];

pub(in crate::lint) fn markdown_lint_issue_codes() -> [MarkdownSyntaxLintCode; 7] {
    [
        MarkdownSyntaxLintCode::UnclosedFrontmatter,
        MarkdownSyntaxLintCode::InvalidFrontmatterYaml,
        MarkdownSyntaxLintCode::UnclosedFence,
        MarkdownSyntaxLintCode::BareObsidianWikilink,
        MarkdownSyntaxLintCode::RedundantObsidianLabel,
        MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink,
        MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder,
    ]
}

pub(in crate::lint) fn markdown_lint_rule_keys() -> Vec<&'static str> {
    let mut keys = markdown_lint_issue_codes()
        .into_iter()
        .map(code_string)
        .collect::<Vec<_>>();
    keys.extend(SYNTHETIC_RULE_KEYS.iter().copied());
    keys
}

pub(in crate::lint) fn code_string(code: MarkdownSyntaxLintCode) -> &'static str {
    match code {
        MarkdownSyntaxLintCode::UnclosedFrontmatter => "unclosed_frontmatter",
        MarkdownSyntaxLintCode::InvalidFrontmatterYaml => "invalid_frontmatter_yaml",
        MarkdownSyntaxLintCode::UnclosedFence => "unclosed_fence",
        MarkdownSyntaxLintCode::BareObsidianWikilink => "bare_obsidian_wikilink",
        MarkdownSyntaxLintCode::RedundantObsidianLabel => "redundant_obsidian_label",
        MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink => "mixed_wikilink_markdown_link",
        MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder => {
            "non_canonical_obsidian_alias_order"
        }
    }
}

pub(super) fn kind_string(kind: MarkdownLintKind) -> &'static str {
    match kind {
        MarkdownLintKind::Syntax => "official_syntax",
        MarkdownLintKind::AuthoringPolicy => "repo_authoring_policy",
    }
}

pub(super) fn bare_or_redundant_expected(metadata: &TargetMetadata) -> String {
    let label = preferred_label(metadata).unwrap_or_else(|| "descriptive label".to_string());
    format!(
        "rewrite as `[[{}|{}]]` or `[{}]({})`.",
        metadata.raw, label, label, metadata.raw
    )
}

pub(super) fn mixed_expected(
    link: Option<&LinkIssueContext>,
    metadata: Option<&TargetMetadata>,
) -> String {
    let Some(metadata) = metadata else {
        return "rewrite as `[[target|label]]` or `[label](target)`.".to_string();
    };
    let label = link
        .and_then(|context| context.label.as_deref())
        .filter(|label| !is_redundant_label_for_target(label, metadata))
        .map(ToOwned::to_owned)
        .or_else(|| preferred_label(metadata))
        .unwrap_or_else(|| "descriptive label".to_string());
    format!(
        "rewrite as `[[{}|{}]]` or `[{}]({})`.",
        metadata.raw, label, label, metadata.raw
    )
}

pub(super) fn non_canonical_expected(
    link: Option<&LinkIssueContext>,
    metadata: Option<&TargetMetadata>,
) -> String {
    let Some(metadata) = metadata else {
        return "rewrite as `[[target|label]]` with the repository target on the left and the display label on the right."
            .to_string();
    };
    let label = link
        .and_then(|context| context.label.as_deref())
        .filter(|label| !is_redundant_label_for_target(label, metadata))
        .map(ToOwned::to_owned)
        .or_else(|| preferred_label(metadata))
        .unwrap_or_else(|| "descriptive label".to_string());
    format!("rewrite as `[[{}|{}]]`.", metadata.raw, label)
}

pub(super) fn display_label_tip(title: Option<&str>, heading: Option<&str>) -> String {
    match (title, heading) {
        (Some(title), Some(heading)) => format!(
            "Target resolves to `{title}` and addresses heading `{heading}`. Let the label carry both note namespace and heading context instead of echoing only the raw target."
        ),
        (Some(title), None) => format!(
            "Target resolves to `{title}`. Reuse that title as the label baseline instead of copying the raw target."
        ),
        (None, Some(heading)) => format!(
            "This target addresses heading `{heading}`. The label should carry note namespace plus heading context, not only the heading text."
        ),
        (None, None) => {
            "Choose a human-readable display label; do not mechanically repeat the raw target."
                .to_string()
        }
    }
}

pub(super) fn normalize_hint(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_space = false;
    for character in value.trim().trim_start_matches('^').chars() {
        let mapped = match character {
            '#' | '-' | '_' | '/' => ' ',
            other => other.to_ascii_lowercase(),
        };
        if mapped.is_ascii_alphanumeric() {
            normalized.push(mapped);
            previous_was_space = false;
        } else if mapped.is_ascii_whitespace() && !previous_was_space && !normalized.is_empty() {
            normalized.push(' ');
            previous_was_space = true;
        }
    }
    normalized.trim().to_string()
}

fn preferred_label(metadata: &TargetMetadata) -> Option<String> {
    match (metadata.title.as_deref(), metadata.heading.as_deref()) {
        (Some(title), Some(heading)) => Some(format!("{title} / {heading}")),
        (Some(title), None) => Some(title.to_string()),
        (None, Some(heading)) => Some(format!("note namespace / {heading}")),
        (None, None) => None,
    }
}

fn is_redundant_label_for_target(label: &str, metadata: &TargetMetadata) -> bool {
    normalize_hint(label) == normalize_hint(metadata.raw.as_str())
        || metadata
            .heading
            .as_deref()
            .is_some_and(|heading| normalize_hint(label) == normalize_hint(heading))
}
