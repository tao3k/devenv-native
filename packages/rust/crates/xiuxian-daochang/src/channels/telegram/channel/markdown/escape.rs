use pulldown_cmark::CodeBlockKind;

pub(super) fn escape_markdown_v2_text(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            match ch {
                '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '='
                | '|' | '{' | '}' | '.' | '!' | '\\' => {
                    escaped.push('\\');
                    escaped.push(ch);
                }
                _ => escaped.push(ch),
            }
            escaped
        })
}

pub(super) fn escape_markdown_v2_code(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            if ch == '\\' || ch == '`' {
                escaped.push('\\');
            }
            escaped.push(ch);
            escaped
        })
}

pub(super) fn escape_markdown_v2_url(url: &str) -> String {
    url.chars()
        .fold(String::with_capacity(url.len()), |mut escaped, ch| {
            if ch == '\\' || ch == ')' {
                escaped.push('\\');
            }
            escaped.push(ch);
            escaped
        })
}

pub(super) fn trim_trailing_blank_lines(text: &mut String) {
    while text.ends_with('\n') {
        text.pop();
    }
}

pub(super) fn normalize_code_fence_language(kind: CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(language) => {
            let normalized = language
                .trim()
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
                .collect::<String>();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        }
    }
}

pub(super) fn escape_html_text(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            match ch {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                _ => escaped.push(ch),
            }
            escaped
        })
}

pub(super) fn escape_html_attr(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut escaped, ch| {
            match ch {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&#39;"),
                _ => escaped.push(ch),
            }
            escaped
        })
}

#[cfg(test)]
#[path = "../../../../../tests/unit/channels/telegram/channel/markdown/escape.rs"]
mod tests;
