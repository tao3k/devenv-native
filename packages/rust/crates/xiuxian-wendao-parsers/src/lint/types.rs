use serde::Serialize;

/// Parser-owned Markdown syntax lint code for lightweight client consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownSyntaxLintCode {
    /// The document is missing the required leading YAML frontmatter block.
    MissingFrontmatter,
    /// The YAML frontmatter exists but does not carry a non-empty `title`.
    MissingFrontmatterTitle,
    /// A skill-shaped document exists but does not carry a non-empty skill name.
    MissingSkillFrontmatterName,
    /// A skill-shaped document exists but does not carry the required
    /// top-level `metadata` mapping.
    MissingSkillFrontmatterMetadata,
    /// The document starts a YAML frontmatter block but never closes it.
    UnclosedFrontmatter,
    /// The YAML frontmatter content is not valid YAML.
    InvalidFrontmatterYaml,
    /// The document starts a fenced code block that never closes.
    UnclosedFence,
    /// The document uses a bare wikilink without an explicit display label.
    BareObsidianWikilink,
    /// The document repeats the wikilink target as the explicit display label.
    RedundantObsidianLabel,
    /// The document mixes wiki-link brackets with Markdown link parentheses.
    MixedWikilinkMarkdownLink,
    /// The document uses a non-canonical Obsidian alias order for path-like targets.
    NonCanonicalObsidianAliasOrder,
}

/// High-level classification for one Markdown lint rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownLintKind {
    /// Violates the official Markdown or Obsidian syntax surface.
    Syntax,
    /// Uses a repo-specific authoring form that is legal syntax but discouraged.
    AuthoringPolicy,
}

impl MarkdownSyntaxLintCode {
    /// Classify whether this rule is official syntax validation or repo policy.
    #[must_use]
    pub const fn kind(self) -> MarkdownLintKind {
        match self {
            Self::UnclosedFrontmatter
            | Self::InvalidFrontmatterYaml
            | Self::UnclosedFence
            | Self::MixedWikilinkMarkdownLink => MarkdownLintKind::Syntax,
            Self::MissingFrontmatter
            | Self::MissingFrontmatterTitle
            | Self::MissingSkillFrontmatterName
            | Self::MissingSkillFrontmatterMetadata
            | Self::BareObsidianWikilink
            | Self::RedundantObsidianLabel
            | Self::NonCanonicalObsidianAliasOrder => MarkdownLintKind::AuthoringPolicy,
        }
    }
}

/// One parser-owned Markdown syntax lint issue with stable positional metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MarkdownSyntaxLintIssue {
    /// Stable machine-readable lint code.
    pub code: MarkdownSyntaxLintCode,
    /// Human-readable diagnostic message.
    pub message: String,
    /// One-based source line.
    pub line: usize,
    /// One-based source column.
    pub column: usize,
}

/// Aggregate parser-owned Markdown syntax lint report for one document body.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct MarkdownSyntaxLintReport {
    /// Stable issue list in source order.
    pub issues: Vec<MarkdownSyntaxLintIssue>,
}

impl MarkdownSyntaxLintReport {
    /// Returns true when the report has no syntax issues.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }
}
