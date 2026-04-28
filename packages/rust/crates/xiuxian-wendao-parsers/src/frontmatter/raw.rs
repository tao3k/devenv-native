use serde_yaml::Value;

/// Borrowed raw frontmatter slice plus the remaining Markdown body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawFrontmatter<'a> {
    /// Raw YAML content without the surrounding fences.
    pub yaml: &'a str,
    /// Remaining Markdown body after the closing fence.
    pub body: &'a str,
}

/// Split one Markdown document into an optional borrowed raw YAML frontmatter
/// slice and the remaining body content.
#[must_use]
pub fn split_frontmatter_raw(content: &str) -> Option<RawFrontmatter<'_>> {
    let opening_len = frontmatter_opening_len(content)?;
    let remainder = &content[opening_len..];
    let mut offset = 0usize;

    while offset <= remainder.len() {
        let line_end = remainder[offset..]
            .find('\n')
            .map_or(remainder.len(), |index| offset + index);
        let next_offset = if line_end < remainder.len() {
            line_end + 1
        } else {
            line_end
        };
        let line = remainder[offset..line_end].trim_end_matches('\r');
        if line == "---" || line == "..." {
            let yaml_end = frontmatter_yaml_end(remainder, offset);
            let body_start = frontmatter_body_start(remainder, next_offset);
            return Some(RawFrontmatter {
                yaml: &remainder[..yaml_end],
                body: &remainder[body_start..],
            });
        }
        if next_offset == offset {
            break;
        }
        offset = next_offset;
    }

    None
}

/// Split one Markdown document into an optional parsed YAML frontmatter value
/// and the remaining body content.
#[must_use]
pub fn split_frontmatter(content: &str) -> (Option<Value>, &str) {
    let Some(parts) = split_frontmatter_raw(content) else {
        return (None, content);
    };
    let parsed = serde_yaml::from_str::<Value>(parts.yaml).ok();
    (parsed, parts.body)
}

fn frontmatter_opening_len(content: &str) -> Option<usize> {
    content
        .strip_prefix("---\n")
        .map(|remainder| content.len() - remainder.len())
        .or_else(|| {
            content
                .strip_prefix("---\r\n")
                .map(|remainder| content.len() - remainder.len())
        })
}

fn frontmatter_yaml_end(remainder: &str, closing_offset: usize) -> usize {
    let yaml = &remainder[..closing_offset];
    if yaml.ends_with("\r\n") {
        closing_offset.saturating_sub(2)
    } else if yaml.ends_with('\n') {
        closing_offset.saturating_sub(1)
    } else {
        closing_offset
    }
}

fn frontmatter_body_start(remainder: &str, closing_next_offset: usize) -> usize {
    let mut offset = closing_next_offset;
    while offset < remainder.len() {
        let line_end = remainder[offset..]
            .find('\n')
            .map_or(remainder.len(), |index| offset + index);
        let next_offset = if line_end < remainder.len() {
            line_end + 1
        } else {
            line_end
        };
        let line = remainder[offset..line_end].trim_end_matches('\r');
        if !line.trim().is_empty() {
            break;
        }
        if next_offset == offset {
            break;
        }
        offset = next_offset;
    }
    offset
}
