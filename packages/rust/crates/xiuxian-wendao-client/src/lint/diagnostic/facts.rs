use super::link::{LinkIssueContext, TargetMetadata};
use super::text::{
    bare_or_redundant_expected, code_string, display_label_tip, kind_string, mixed_expected,
    non_canonical_expected,
};
use std::path::Path;
use xiuxian_wendao_parsers::{MarkdownSyntaxLintCode, MarkdownSyntaxLintIssue};

pub(in crate::lint) struct DiagnosticFacts {
    rule_key: String,
    kind: String,
    parser_code: Option<MarkdownSyntaxLintCode>,
    parser_message: Option<String>,
    line: usize,
    column: usize,
    source: Option<String>,
    link: Option<LinkIssueContext>,
    target_metadata: Option<TargetMetadata>,
    duplicates_heading: bool,
    utf8_error: Option<String>,
    problem_override: Option<String>,
    detail_override: Option<String>,
    found_override: Option<String>,
    expected_override: Option<String>,
    tip_override: Option<String>,
}

pub(in crate::lint) struct DynamicDiagnosticText {
    pub(in crate::lint) problem: String,
    pub(in crate::lint) detail: String,
    pub(in crate::lint) found: Option<String>,
    pub(in crate::lint) expected: Option<String>,
    pub(in crate::lint) tip: Option<String>,
}

#[derive(Clone, Copy)]
pub(in crate::lint) struct LocalTargetScopeViolation<'a> {
    pub(in crate::lint) resolved_path: &'a Path,
    pub(in crate::lint) lint_root: &'a Path,
}

#[derive(Clone, Copy)]
pub(in crate::lint) struct LocalTargetTransientViolation<'a> {
    pub(in crate::lint) resolved_path: &'a Path,
    pub(in crate::lint) lint_root: &'a Path,
    pub(in crate::lint) offending_dir: &'a str,
}

pub(in crate::lint) struct LocalTargetFragmentViolation<'a> {
    pub(in crate::lint) literal: &'a str,
    pub(in crate::lint) raw_target: &'a str,
    pub(in crate::lint) fragment: &'a str,
    pub(in crate::lint) is_block: bool,
    pub(in crate::lint) target_title: Option<String>,
}

impl DiagnosticFacts {
    pub(super) fn from_parser_issue(
        issue: MarkdownSyntaxLintIssue,
        source: Option<String>,
        link: Option<LinkIssueContext>,
        target_metadata: Option<TargetMetadata>,
        duplicates_heading: bool,
    ) -> Self {
        Self {
            rule_key: code_string(issue.code).to_string(),
            kind: kind_string(issue.code.kind()).to_string(),
            parser_code: Some(issue.code),
            parser_message: Some(issue.message),
            line: issue.line,
            column: issue.column,
            source,
            link,
            target_metadata,
            duplicates_heading,
            utf8_error: None,
            problem_override: None,
            detail_override: None,
            found_override: None,
            expected_override: None,
            tip_override: None,
        }
    }

    pub(in crate::lint) fn invalid_utf8(error: String) -> Self {
        Self {
            rule_key: "invalid_utf8".to_string(),
            kind: "official_syntax".to_string(),
            parser_code: None,
            parser_message: None,
            line: 1,
            column: 1,
            source: None,
            link: None,
            target_metadata: None,
            duplicates_heading: false,
            utf8_error: Some(error),
            problem_override: None,
            detail_override: None,
            found_override: None,
            expected_override: None,
            tip_override: None,
        }
    }

    pub(in crate::lint) fn missing_local_target(
        relative_path: &str,
        line: usize,
        column: usize,
        source: Option<String>,
        literal: &str,
        raw_target: &str,
    ) -> Self {
        Self {
            rule_key: "missing_local_target".to_string(),
            kind: "repo_authoring_policy".to_string(),
            parser_code: None,
            parser_message: None,
            line,
            column,
            source,
            link: Some(LinkIssueContext {
                literal: literal.to_string(),
                target: raw_target.to_string(),
                label: None,
            }),
            target_metadata: None,
            duplicates_heading: false,
            utf8_error: None,
            problem_override: Some(
                "Referenced local link or attachment target does not exist.".to_string(),
            ),
            detail_override: Some(format!(
                "Target `{raw_target}` in `{relative_path}` did not resolve to an existing local note or attachment. Resolution checks the source directory, parent directories up to the lint root, plus `.md` and `index.md` fallbacks for note-like targets."
            )),
            found_override: None,
            expected_override: Some(
                "Fix the target path, create the missing local file, or remove the broken reference."
                    .to_string(),
            ),
            tip_override: Some(
                "This rule only checks whether the local file resolved; addressed headings and block anchors are validated separately when present. External URLs are ignored.".to_string(),
            ),
        }
    }

    pub(in crate::lint) fn missing_local_fragment(
        relative_path: &str,
        line: usize,
        column: usize,
        source: Option<String>,
        fragment_violation: LocalTargetFragmentViolation<'_>,
    ) -> Self {
        let fragment_kind = if fragment_violation.is_block {
            "block anchor"
        } else {
            "heading anchor"
        };
        let problem = format!("Referenced local {fragment_kind} does not exist.");
        let detail = match fragment_violation.target_title.as_deref() {
            Some(title) => format!(
                "Target `{}` in `{relative_path}` resolved to `{title}`, but the addressed {fragment_kind} `{}` was not found.",
                fragment_violation.raw_target, fragment_violation.fragment
            ),
            None => format!(
                "Target `{}` in `{relative_path}` resolved to an existing local note, but the addressed {fragment_kind} `{}` was not found.",
                fragment_violation.raw_target, fragment_violation.fragment
            ),
        };
        let expected = if fragment_violation.is_block {
            "Fix the block fragment, or add the missing `^block-id` anchor to the target note."
        } else {
            "Fix the heading fragment, or add the missing heading to the target note."
        };
        Self {
            rule_key: "missing_local_fragment".to_string(),
            kind: "repo_authoring_policy".to_string(),
            parser_code: None,
            parser_message: None,
            line,
            column,
            source,
            link: Some(LinkIssueContext {
                literal: fragment_violation.literal.to_string(),
                target: fragment_violation.raw_target.to_string(),
                label: None,
            }),
            target_metadata: Some(TargetMetadata {
                raw: fragment_violation.raw_target.to_string(),
                heading: Some(fragment_violation.fragment.to_string()),
                title: fragment_violation.target_title,
            }),
            duplicates_heading: false,
            utf8_error: None,
            problem_override: Some(problem),
            detail_override: Some(detail),
            found_override: None,
            expected_override: Some(expected.to_string()),
            tip_override: Some(
                "This rule runs only after local path resolution succeeds and then validates the addressed heading or block anchor inside the target note.".to_string(),
            ),
        }
    }

    pub(in crate::lint) fn local_target_outside_root(
        relative_path: &str,
        line: usize,
        column: usize,
        source: Option<String>,
        literal: &str,
        raw_target: &str,
        scope_violation: LocalTargetScopeViolation<'_>,
    ) -> Self {
        let resolved = scope_violation.resolved_path.to_string_lossy();
        let root = scope_violation.lint_root.to_string_lossy();
        Self {
            rule_key: "local_target_outside_root".to_string(),
            kind: "repo_authoring_policy".to_string(),
            parser_code: None,
            parser_message: None,
            line,
            column,
            source,
            link: Some(LinkIssueContext {
                literal: literal.to_string(),
                target: raw_target.to_string(),
                label: None,
            }),
            target_metadata: None,
            duplicates_heading: false,
            utf8_error: None,
            problem_override: Some(
                "Referenced local link or attachment escapes the active lint root.".to_string(),
            ),
            detail_override: Some(format!(
                "Target `{raw_target}` in `{relative_path}` resolved to `{resolved}`, which is outside the active lint root `{root}`. Local Markdown targets must stay within the current repo-local lint scope."
            )),
            found_override: None,
            expected_override: Some(
                "Retarget the link to an in-root path, or widen the lint root intentionally before relying on this reference."
                    .to_string(),
            ),
            tip_override: Some(
                "Do not use `..` traversal to escape the active lint root for local notes or attachments."
                    .to_string(),
            ),
        }
    }

    pub(in crate::lint) fn local_target_transient_dir(
        relative_path: &str,
        line: usize,
        column: usize,
        source: Option<String>,
        literal: &str,
        raw_target: &str,
        transient_violation: LocalTargetTransientViolation<'_>,
    ) -> Self {
        let resolved = transient_violation.resolved_path.to_string_lossy();
        let root = transient_violation.lint_root.to_string_lossy();
        let offending_dir = transient_violation.offending_dir;
        Self {
            rule_key: "local_target_transient_dir".to_string(),
            kind: "repo_authoring_policy".to_string(),
            parser_code: None,
            parser_message: None,
            line,
            column,
            source,
            link: Some(LinkIssueContext {
                literal: literal.to_string(),
                target: raw_target.to_string(),
                label: None,
            }),
            target_metadata: None,
            duplicates_heading: false,
            utf8_error: None,
            problem_override: Some(
                "Referenced local link or attachment points into a transient or generated repository directory.".to_string(),
            ),
            detail_override: Some(format!(
                "Target `{raw_target}` in `{relative_path}` resolved to `{resolved}` under lint root `{root}`, but it passes through transient/generated directory `{offending_dir}`. These directories are operational surfaces, not stable authoring targets."
            )),
            found_override: None,
            expected_override: Some(
                "Retarget the link to stable repository content outside transient/generated directories, or move the referenced artifact into a governed source directory."
                    .to_string(),
            ),
            tip_override: Some(
                "Directories such as `.cache`, `.data`, `.run`, `.config`, `.bin`, `target`, and `node_modules` are treated as unstable authoring surfaces."
                    .to_string(),
            ),
        }
    }

    pub(in crate::lint) fn directory_link_style_policy(
        rule_key: String,
        line: usize,
        column: usize,
        source: Option<String>,
        dynamic_text: DynamicDiagnosticText,
    ) -> Self {
        Self {
            rule_key,
            kind: "repo_authoring_policy".to_string(),
            parser_code: None,
            parser_message: None,
            line,
            column,
            source,
            link: None,
            target_metadata: None,
            duplicates_heading: false,
            utf8_error: None,
            problem_override: Some(dynamic_text.problem),
            detail_override: Some(dynamic_text.detail),
            found_override: dynamic_text.found,
            expected_override: dynamic_text.expected,
            tip_override: dynamic_text.tip,
        }
    }

    pub(in crate::lint) fn rule_key(&self) -> &str {
        self.rule_key.as_str()
    }

    pub(in crate::lint) fn kind(&self) -> &str {
        self.kind.as_str()
    }

    pub(in crate::lint) fn parser_message(&self) -> &str {
        match self.parser_message.as_deref() {
            Some(message) => message,
            None => panic!("parser-backed diagnostic facts should carry a parser message"),
        }
    }

    pub(in crate::lint) fn utf8_error(&self) -> &str {
        match self.utf8_error.as_deref() {
            Some(error) => error,
            None => panic!("invalid_utf8 diagnostic facts should carry a utf8 error"),
        }
    }

    pub(in crate::lint) fn line(&self) -> usize {
        self.line
    }

    pub(in crate::lint) fn column(&self) -> usize {
        self.column
    }

    pub(in crate::lint) fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub(in crate::lint) fn target(&self) -> Option<&str> {
        self.link.as_ref().map(|context| context.target.as_str())
    }

    pub(in crate::lint) fn target_title(&self) -> Option<&str> {
        self.target_metadata
            .as_ref()
            .and_then(|metadata| metadata.title.as_deref())
    }

    pub(in crate::lint) fn target_heading(&self) -> Option<&str> {
        self.target_metadata
            .as_ref()
            .and_then(|metadata| metadata.heading.as_deref())
    }

    pub(in crate::lint) fn link_literal(&self) -> Option<&str> {
        self.link.as_ref().map(|context| context.literal.as_str())
    }

    pub(in crate::lint) fn rewrite_with_markdown(&self) -> Option<String> {
        match self.parser_code {
            Some(
                MarkdownSyntaxLintCode::BareObsidianWikilink
                | MarkdownSyntaxLintCode::RedundantObsidianLabel,
            ) => self
                .target_metadata
                .as_ref()
                .map(bare_or_redundant_expected),
            Some(MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink) => Some(mixed_expected(
                self.link.as_ref(),
                self.target_metadata.as_ref(),
            )),
            Some(
                MarkdownSyntaxLintCode::MissingFrontmatter
                | MarkdownSyntaxLintCode::MissingFrontmatterTitle
                | MarkdownSyntaxLintCode::MissingFrontmatterKind
                | MarkdownSyntaxLintCode::MissingFrontmatterCategory
                | MarkdownSyntaxLintCode::MissingFrontmatterTags
                | MarkdownSyntaxLintCode::MissingFrontmatterDescription
                | MarkdownSyntaxLintCode::MissingFrontmatterAuthor
                | MarkdownSyntaxLintCode::MissingFrontmatterDate
                | MarkdownSyntaxLintCode::InvalidFrontmatterDatePrecision
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterName
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
                | MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
                | MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder
                | MarkdownSyntaxLintCode::UnclosedFrontmatter
                | MarkdownSyntaxLintCode::InvalidFrontmatterYaml
                | MarkdownSyntaxLintCode::UnclosedFence,
            )
            | None => None,
        }
    }

    pub(in crate::lint) fn rewrite_wikilink_only(&self) -> Option<String> {
        match self.parser_code {
            Some(MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder) => Some(
                non_canonical_expected(self.link.as_ref(), self.target_metadata.as_ref()),
            ),
            Some(
                MarkdownSyntaxLintCode::MissingFrontmatter
                | MarkdownSyntaxLintCode::MissingFrontmatterTitle
                | MarkdownSyntaxLintCode::MissingFrontmatterKind
                | MarkdownSyntaxLintCode::MissingFrontmatterCategory
                | MarkdownSyntaxLintCode::MissingFrontmatterTags
                | MarkdownSyntaxLintCode::MissingFrontmatterDescription
                | MarkdownSyntaxLintCode::MissingFrontmatterAuthor
                | MarkdownSyntaxLintCode::MissingFrontmatterDate
                | MarkdownSyntaxLintCode::InvalidFrontmatterDatePrecision
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterName
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
                | MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
                | MarkdownSyntaxLintCode::UnclosedFrontmatter
                | MarkdownSyntaxLintCode::InvalidFrontmatterYaml
                | MarkdownSyntaxLintCode::UnclosedFence
                | MarkdownSyntaxLintCode::BareObsidianWikilink
                | MarkdownSyntaxLintCode::RedundantObsidianLabel
                | MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink,
            )
            | None => None,
        }
    }

    pub(in crate::lint) fn display_label_tip(&self) -> Option<String> {
        match self.parser_code {
            Some(
                MarkdownSyntaxLintCode::BareObsidianWikilink
                | MarkdownSyntaxLintCode::RedundantObsidianLabel
                | MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink
                | MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder,
            ) => Some(display_label_tip(
                self.target_title(),
                self.target_heading(),
            )),
            Some(
                MarkdownSyntaxLintCode::MissingFrontmatter
                | MarkdownSyntaxLintCode::MissingFrontmatterTitle
                | MarkdownSyntaxLintCode::MissingFrontmatterKind
                | MarkdownSyntaxLintCode::MissingFrontmatterCategory
                | MarkdownSyntaxLintCode::MissingFrontmatterTags
                | MarkdownSyntaxLintCode::MissingFrontmatterDescription
                | MarkdownSyntaxLintCode::MissingFrontmatterAuthor
                | MarkdownSyntaxLintCode::MissingFrontmatterDate
                | MarkdownSyntaxLintCode::InvalidFrontmatterDatePrecision
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalSaliencyBase
                | MarkdownSyntaxLintCode::MissingFrontmatterRetrievalDecayRate
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterName
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
                | MarkdownSyntaxLintCode::InvalidSkillFrontmatterSchema
                | MarkdownSyntaxLintCode::UnclosedFrontmatter
                | MarkdownSyntaxLintCode::InvalidFrontmatterYaml
                | MarkdownSyntaxLintCode::UnclosedFence,
            )
            | None => None,
        }
    }

    pub(in crate::lint) fn redundant_problem(&self) -> String {
        if self.duplicates_heading {
            "Obsidian officially allows this heading display text, but repository authoring policy treats it as redundant because it repeats the addressed heading."
                .to_string()
        } else {
            "Obsidian officially allows this display text, but repository authoring policy treats it as redundant because it repeats the raw target."
                .to_string()
        }
    }

    pub(in crate::lint) fn dynamic_problem_text(&self) -> &str {
        match self.problem_override.as_deref() {
            Some(problem) => problem,
            None => panic!("dynamic problem text should exist"),
        }
    }

    pub(in crate::lint) fn dynamic_detail_text(&self) -> &str {
        match self.detail_override.as_deref() {
            Some(detail) => detail,
            None => panic!("dynamic detail text should exist"),
        }
    }

    pub(in crate::lint) fn dynamic_found_text(&self) -> Option<String> {
        self.found_override.clone()
    }

    pub(in crate::lint) fn dynamic_expected_text(&self) -> Option<String> {
        self.expected_override.clone()
    }

    pub(in crate::lint) fn dynamic_tip_text(&self) -> Option<String> {
        self.tip_override.clone()
    }
}
