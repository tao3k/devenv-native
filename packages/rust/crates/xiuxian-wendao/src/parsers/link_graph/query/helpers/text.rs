//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
pub(in crate::parsers::link_graph::query) fn split_terms_preserving_quotes(
    raw: &str,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in raw.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                out.push(current.clone());
                current.clear();
            }
            continue;
        }
        current.push(ch);
    }

    if !current.is_empty() {
        out.push(current);
    }
    out
}

pub(in crate::parsers::link_graph::query) fn normalize_boolean_operators(raw: &str) -> String {
    let mut text = raw.trim().replace("||", "|").replace("&&", "&");
    for (from, to) in [
        (" OR ", " | "),
        (" and ", " & "),
        (" AND ", " & "),
        (" or ", " | "),
        (" not ", " !"),
        (" NOT ", " !"),
    ] {
        text = text.replace(from, to);
    }
    text
}

pub(in crate::parsers::link_graph::query) fn split_top_level(
    raw: &str,
    separators: &[char],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut depth: i32 = 0;

    for ch in raw.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }

        if ch == '(' {
            depth += 1;
            current.push(ch);
            continue;
        }
        if ch == ')' {
            depth = (depth - 1).max(0);
            current.push(ch);
            continue;
        }

        if depth == 0 && separators.contains(&ch) {
            let token = current.trim();
            if !token.is_empty() {
                out.push(token.to_string());
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    let token = current.trim();
    if !token.is_empty() {
        out.push(token.to_string());
    }
    out
}

pub(in crate::parsers::link_graph::query) fn has_balanced_outer_parens(raw: &str) -> bool {
    let text = raw.trim();
    if text.len() < 2 || !text.starts_with('(') || !text.ends_with(')') {
        return false;
    }
    let mut depth: i32 = 0;
    let mut quote: Option<char> = None;
    for (idx, ch) in text.char_indices() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth < 0 {
                return false;
            }
            if depth == 0 && idx < text.len() - 1 {
                return false;
            }
        }
    }
    depth == 0
}

pub(in crate::parsers::link_graph::query) fn strip_outer_parens(raw: &str) -> String {
    let mut text = raw.trim().to_string();
    while has_balanced_outer_parens(&text) {
        text = text[1..text.len().saturating_sub(1)].trim().to_string();
    }
    text
}

pub(in crate::parsers::link_graph::query) fn paren_balance(raw: &str) -> i32 {
    let mut depth: i32 = 0;
    let mut quote: Option<char> = None;
    for ch in raw.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
        }
    }
    depth
}

pub(in crate::parsers::link_graph::query) fn is_boolean_connector_token(raw: &str) -> bool {
    let token = raw.trim();
    token == "&"
        || token == "|"
        || token.eq_ignore_ascii_case("and")
        || token.eq_ignore_ascii_case("or")
}
