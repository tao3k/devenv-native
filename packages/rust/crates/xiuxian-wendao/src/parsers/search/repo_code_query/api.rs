use super::ParsedRepoCodeSearchQuery;

/// Prefixes that select backend-owned filters in repository code search queries.
pub const REPO_CODE_SEARCH_BACKEND_PREFIXES: &[&str] = &["lang", "kind", "repo"];
/// Prefixes that select structural parser filters in repository code search queries.
pub const REPO_CODE_SEARCH_STRUCTURAL_PREFIXES: &[&str] = &["ast", "sg"];
/// Supported values for the `kind:` repository code search filter.
pub const REPO_CODE_SEARCH_KIND_FILTER_VALUES: &[&str] =
    &["file", "symbol", "function", "module", "example"];
/// Accepted aliases for canonical repository code search prefixes.
pub const REPO_CODE_SEARCH_PREFIX_ALIASES: &[(&str, &str)] = &[("language", "lang")];

/// Parse a user-facing repository code search query into typed filter state.
#[must_use]
pub fn parse_repo_code_search_query(query: &str) -> ParsedRepoCodeSearchQuery {
    parse_repo_code_search_query_with_repo_hint(query, None)
}

/// Parse a repository code search query while applying a default repository hint.
pub fn parse_repo_code_search_query_with_repo_hint(
    query: &str,
    repo_hint: Option<&str>,
) -> ParsedRepoCodeSearchQuery {
    let mut spec = ParsedRepoCodeSearchQuery {
        repo: repo_hint
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        ..ParsedRepoCodeSearchQuery::default()
    };
    let mut search_tokens = Vec::new();

    for token in tokenize_repo_code_search_query(query) {
        if let Some(value) = token.strip_prefix("lang:") {
            let normalized = value.trim().to_ascii_lowercase();
            if !normalized.is_empty() {
                spec.language_filters.insert(normalized);
            }
            continue;
        }

        if let Some(value) = token.strip_prefix("repo:") {
            let repo_id = value.trim();
            if !repo_id.is_empty() {
                spec.repo = Some(repo_id.to_string());
            }
            continue;
        }

        if let Some(value) = token.strip_prefix("kind:") {
            let normalized = value.trim().to_ascii_lowercase();
            if REPO_CODE_SEARCH_KIND_FILTER_VALUES.contains(&normalized.as_str()) {
                spec.kind_filters.insert(normalized);
                continue;
            }
        }

        if let Some(value) = token
            .strip_prefix("ast:")
            .or_else(|| token.strip_prefix("sg:"))
        {
            let normalized = strip_matching_quotes(value.trim());
            if !normalized.is_empty() {
                spec.ast_pattern = Some(normalized);
            }
            continue;
        }

        search_tokens.push(token.clone());
    }

    spec.search_term = (!search_tokens.is_empty()).then(|| search_tokens.join(" "));
    spec
}

fn tokenize_repo_code_search_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut active_quote = None;

    for character in query.chars() {
        match active_quote {
            Some(quote) => {
                current.push(character);
                if character == quote {
                    active_quote = None;
                }
            }
            None if character.is_ascii_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => {
                if matches!(character, '"' | '\'') {
                    active_quote = Some(character);
                }
                current.push(character);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn strip_matching_quotes(value: &str) -> String {
    if value.len() >= 2 {
        let mut characters = value.chars();
        if let (Some(start), Some(end)) = (characters.next(), value.chars().last())
            && start == end
            && matches!(start, '"' | '\'')
        {
            return characters
                .take(value.chars().count().saturating_sub(2))
                .collect();
        }
    }

    value.to_string()
}
