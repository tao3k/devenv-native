use xiuxian_wendao_parsers::MarkdownSyntaxLintCode;

#[derive(Clone)]
pub(super) struct LinkIssueContext {
    pub(super) literal: String,
    pub(super) target: String,
    pub(super) label: Option<String>,
}

pub(super) struct TargetMetadata {
    pub(super) raw: String,
    pub(super) heading: Option<String>,
    pub(super) title: Option<String>,
}

impl LinkIssueContext {
    pub(super) fn from_source(
        code: MarkdownSyntaxLintCode,
        line: &str,
        column: usize,
    ) -> Option<Self> {
        match code {
            MarkdownSyntaxLintCode::BareObsidianWikilink
            | MarkdownSyntaxLintCode::RedundantObsidianLabel
            | MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder => {
                parse_wikilink_issue_context(code, line, column)
            }
            MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink => {
                parse_mixed_issue_context(line, column)
            }
            MarkdownSyntaxLintCode::MissingFrontmatter
            | MarkdownSyntaxLintCode::MissingFrontmatterTitle
            | MarkdownSyntaxLintCode::MissingSkillFrontmatterName
            | MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
            | MarkdownSyntaxLintCode::UnclosedFrontmatter
            | MarkdownSyntaxLintCode::InvalidFrontmatterYaml
            | MarkdownSyntaxLintCode::UnclosedFence => None,
        }
    }
}

pub(super) fn split_target_path_and_heading(raw_target: &str) -> (String, Option<String>) {
    let (path, fragment) = split_target_path_and_fragment(raw_target);
    (path, fragment.as_deref().and_then(normalize_heading))
}

pub(super) fn split_target_path_and_fragment(raw_target: &str) -> (String, Option<String>) {
    let trimmed = raw_target.trim();
    match trimmed.split_once('#') {
        Some((path, fragment)) => {
            let fragment = fragment.trim();
            (
                path.trim().to_string(),
                (!fragment.is_empty()).then(|| fragment.to_string()),
            )
        }
        None => (trimmed.to_string(), None),
    }
}

fn parse_wikilink_issue_context(
    code: MarkdownSyntaxLintCode,
    line: &str,
    column: usize,
) -> Option<LinkIssueContext> {
    let rest = slice_from_column(line, column)?;
    let (embed_prefix, rest) = rest
        .strip_prefix('!')
        .map_or(("", rest), |_| ("!", &rest[1..]));
    let inner = rest.strip_prefix("[[")?;
    let close = inner.find("]]")?;
    let literal = format!("{embed_prefix}[[{}]]", &inner[..close]);
    let inner = &inner[..close];
    match inner.split_once('|') {
        Some((left, right)) => {
            let left = left.trim();
            let right = right.trim();
            match code {
                MarkdownSyntaxLintCode::NonCanonicalObsidianAliasOrder => Some(LinkIssueContext {
                    literal,
                    target: right.to_string(),
                    label: Some(left.to_string()),
                }),
                MarkdownSyntaxLintCode::RedundantObsidianLabel
                | MarkdownSyntaxLintCode::BareObsidianWikilink
                | MarkdownSyntaxLintCode::MixedWikilinkMarkdownLink
                | MarkdownSyntaxLintCode::MissingFrontmatter
                | MarkdownSyntaxLintCode::MissingFrontmatterTitle
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterName
                | MarkdownSyntaxLintCode::MissingSkillFrontmatterMetadata
                | MarkdownSyntaxLintCode::UnclosedFrontmatter
                | MarkdownSyntaxLintCode::InvalidFrontmatterYaml
                | MarkdownSyntaxLintCode::UnclosedFence => Some(LinkIssueContext {
                    literal,
                    target: left.to_string(),
                    label: Some(right.to_string()),
                }),
            }
        }
        None => Some(LinkIssueContext {
            literal,
            target: inner.trim().to_string(),
            label: None,
        }),
    }
}

fn parse_mixed_issue_context(line: &str, column: usize) -> Option<LinkIssueContext> {
    let rest = slice_from_column(line, column)?;
    let (embed_prefix, rest) = rest
        .strip_prefix('!')
        .map_or(("", rest), |_| ("!", &rest[1..]));
    let inner = rest.strip_prefix("[[")?;
    let close = inner.find("]]")?;
    let after = &inner[(close + 2)..];
    let after = after.strip_prefix('(')?;
    let markdown_close = after.find(')')?;
    let wiki_inner = inner[..close].trim();
    let (target, _wikilink_label) = wiki_inner
        .split_once('|')
        .map_or((wiki_inner, None), |(left, right)| {
            (left.trim(), Some(right.trim()))
        });
    Some(LinkIssueContext {
        literal: format!(
            "{embed_prefix}[[{}]]({})",
            wiki_inner,
            &after[..markdown_close]
        ),
        target: target.to_string(),
        label: Some(after[..markdown_close].trim().to_string()),
    })
}

fn normalize_heading(heading: &str) -> Option<String> {
    let heading = heading.trim().trim_start_matches('^').trim();
    if heading.is_empty() {
        None
    } else {
        Some(heading.to_string())
    }
}

fn slice_from_column(line: &str, column: usize) -> Option<&str> {
    let byte_index = if column <= 1 {
        0
    } else {
        line.char_indices()
            .nth(column - 1)
            .map_or(line.len(), |(index, _)| index)
    };
    line.get(byte_index..)
}
