use super::link::{LinkIssueContext, TargetMetadata};
use super::text::{
    bare_or_redundant_expected, code_string, display_label_tip, kind_string, mixed_expected,
    non_canonical_expected,
};
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

    pub(in crate::lint) fn directory_link_style_policy(
        rule_key: String,
        line: usize,
        column: usize,
        source: Option<String>,
        problem: String,
        detail: String,
        found: Option<String>,
        expected: Option<String>,
        tip: Option<String>,
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
            problem_override: Some(problem),
            detail_override: Some(detail),
            found_override: found,
            expected_override: expected,
            tip_override: tip,
        }
    }

    pub(in crate::lint) fn rule_key(&self) -> &str {
        self.rule_key.as_str()
    }

    pub(in crate::lint) fn kind(&self) -> &str {
        self.kind.as_str()
    }

    pub(in crate::lint) fn parser_message(&self) -> &str {
        self.parser_message
            .as_deref()
            .expect("parser-backed diagnostic facts should carry a parser message")
    }

    pub(in crate::lint) fn utf8_error(&self) -> &str {
        self.utf8_error
            .as_deref()
            .expect("invalid_utf8 diagnostic facts should carry a utf8 error")
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
            Some(MarkdownSyntaxLintCode::BareObsidianWikilink)
            | Some(MarkdownSyntaxLintCode::RedundantObsidianLabel) => self
                .target_metadata
                .as_ref()
                .map(bare_or_redundant_expected),
            Some(MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink) => Some(mixed_expected(
                self.link.as_ref(),
                self.target_metadata.as_ref(),
            )),
            Some(MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder)
            | Some(MarkdownSyntaxLintCode::UnclosedFrontmatter)
            | Some(MarkdownSyntaxLintCode::InvalidFrontmatterYaml)
            | Some(MarkdownSyntaxLintCode::UnclosedFence)
            | None => None,
        }
    }

    pub(in crate::lint) fn rewrite_wikilink_only(&self) -> Option<String> {
        match self.parser_code {
            Some(MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder) => Some(
                non_canonical_expected(self.link.as_ref(), self.target_metadata.as_ref()),
            ),
            Some(MarkdownSyntaxLintCode::UnclosedFrontmatter)
            | Some(MarkdownSyntaxLintCode::InvalidFrontmatterYaml)
            | Some(MarkdownSyntaxLintCode::UnclosedFence)
            | Some(MarkdownSyntaxLintCode::BareObsidianWikilink)
            | Some(MarkdownSyntaxLintCode::RedundantObsidianLabel)
            | Some(MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink)
            | None => None,
        }
    }

    pub(in crate::lint) fn display_label_tip(&self) -> Option<String> {
        match self.parser_code {
            Some(MarkdownSyntaxLintCode::BareObsidianWikilink)
            | Some(MarkdownSyntaxLintCode::RedundantObsidianLabel)
            | Some(MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink)
            | Some(MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder) => Some(
                display_label_tip(self.target_title(), self.target_heading()),
            ),
            Some(MarkdownSyntaxLintCode::UnclosedFrontmatter)
            | Some(MarkdownSyntaxLintCode::InvalidFrontmatterYaml)
            | Some(MarkdownSyntaxLintCode::UnclosedFence)
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
        self.problem_override
            .as_deref()
            .expect("dynamic problem text should exist")
    }

    pub(in crate::lint) fn dynamic_detail_text(&self) -> &str {
        self.detail_override
            .as_deref()
            .expect("dynamic detail text should exist")
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
