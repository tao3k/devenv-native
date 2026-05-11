//! Minimal glob-like matching for code observation path scopes.

/// Find the closing quote in a string, handling escaped quotes.
pub(super) fn find_closing_quote(s: &str) -> Option<usize> {
    let mut chars = s.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if ch == '\\' {
            chars.next();
            continue;
        }
        if ch == '"' {
            return Some(index);
        }
    }

    None
}

/// Check if a file path matches a scope glob pattern.
///
/// # Supported Patterns
///
/// - `**` matches any number of directories
/// - `*` matches any single path segment
/// - `?` matches any single character
///
/// # Examples
///
/// ```
/// use xiuxian_wendao_parsers::path_matches_scope;
///
/// assert!(path_matches_scope("src/api/handler.rs", "src/api/**"));
/// assert!(path_matches_scope("src/api/handler.rs", "**/*.rs"));
/// assert!(!path_matches_scope("src/api/handler.rs", "src/db/**"));
/// ```
#[must_use]
pub fn path_matches_scope(file_path: &str, scope: &str) -> bool {
    let normalized_path = file_path.replace('\\', "/");
    let normalized_scope = scope.replace('\\', "/");

    if normalized_scope.contains("**") {
        match_glob_with_double_star(&normalized_path, &normalized_scope)
    } else {
        match_simple_glob(&normalized_path, &normalized_scope)
    }
}

fn match_glob_with_double_star(path: &str, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split("**").collect();

    if parts.is_empty() {
        return true;
    }

    if !parts[0].is_empty() && !path.starts_with(parts[0]) {
        return false;
    }

    if let Some(last) = parts
        .last()
        .filter(|last| parts.len() > 1 && !last.is_empty())
    {
        if let Some(trailing_pattern) = last.strip_prefix('/') {
            if !path.ends_with(trailing_pattern) {
                if trailing_pattern.contains('*') || trailing_pattern.contains('?') {
                    if let Some(slash_pos) = path.rfind('/') {
                        let filename = &path[slash_pos + 1..];
                        if !match_simple_glob(filename, trailing_pattern) {
                            return false;
                        }
                    } else if !match_simple_glob(path, trailing_pattern) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        } else if !path.ends_with(last) {
            let remaining = if parts[0].is_empty() {
                path
            } else {
                &path[parts[0].len()..]
            };
            if !remaining.contains(last) && !remaining.ends_with(last) {
                return false;
            }
        }
    }

    let mut search_pos = 0;
    for (index, part) in parts.iter().enumerate() {
        if index == 0 || index == parts.len() - 1 || part.is_empty() {
            continue;
        }

        let part = if let Some(stripped) = part.strip_prefix('/') {
            stripped
        } else {
            *part
        };

        if search_pos >= path.len() {
            return false;
        }

        if let Some(position) = path[search_pos..].find(part) {
            search_pos += position + part.len();
        } else {
            return false;
        }
    }

    true
}

fn match_simple_glob(path: &str, pattern: &str) -> bool {
    let path_chars: Vec<char> = path.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut path_index = 0;
    let mut pattern_index = 0;

    while pattern_index < pattern_chars.len() {
        let pattern_char = pattern_chars[pattern_index];

        if pattern_char == '*' {
            let next_pattern_index = next_non_star_pattern_index(&pattern_chars, pattern_index);
            return glob_star_matches(
                &path_chars,
                path_index,
                &pattern_chars[next_pattern_index..],
            );
        }

        if pattern_char == '?' {
            if path_index >= path_chars.len() || path_chars[path_index] == '/' {
                return false;
            }
            path_index += 1;
            pattern_index += 1;
            continue;
        }

        if path_index >= path_chars.len() || path_chars[path_index] != pattern_char {
            return false;
        }

        path_index += 1;
        pattern_index += 1;
    }

    path_index == path_chars.len()
}

fn next_non_star_pattern_index(pattern_chars: &[char], pattern_index: usize) -> usize {
    pattern_chars[pattern_index..]
        .iter()
        .take_while(|character| **character == '*')
        .count()
        + pattern_index
}

fn glob_star_matches(path_chars: &[char], path_index: usize, remaining_pattern: &[char]) -> bool {
    let remaining_pattern_str = chars_to_string(remaining_pattern);
    let max_match_len = path_chars[path_index..]
        .iter()
        .take_while(|&&ch| ch != '/')
        .count();

    (0..=max_match_len).any(|try_len| {
        let remaining_path = chars_to_string(&path_chars[path_index + try_len..]);
        match_simple_glob(&remaining_path, &remaining_pattern_str)
    })
}

fn chars_to_string(chars: &[char]) -> String {
    chars.iter().collect()
}
