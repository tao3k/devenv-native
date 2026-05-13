//! Parser for user-facing repository code search query filters.

use super::types::ParsedRepoCodeSearchQuery;

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
#[must_use]
pub fn parse_repo_code_search_query_with_repo_hint(
    query: &str,
    repo_hint: Option<&str>,
) -> ParsedRepoCodeSearchQuery {
    let mut spec = query_spec_with_repo_hint(repo_hint);
    let mut search_tokens = Vec::new();

    for token in tokenize_repo_code_search_query(query) {
        apply_repo_code_search_token(&mut spec, &mut search_tokens, token.as_str());
    }

    spec.search_term = search_term_from_tokens(&search_tokens);
    spec
}

enum RepoCodeSearchToken<'a> {
    Language(&'a str),
    Repo(&'a str),
    Kind(&'a str),
    AstPattern(&'a str),
    Search(&'a str),
}

fn query_spec_with_repo_hint(repo_hint: Option<&str>) -> ParsedRepoCodeSearchQuery {
    ParsedRepoCodeSearchQuery {
        repo: normalized_repo_hint(repo_hint),
        ..ParsedRepoCodeSearchQuery::default()
    }
}

fn normalized_repo_hint(repo_hint: Option<&str>) -> Option<String> {
    repo_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn apply_repo_code_search_token(
    spec: &mut ParsedRepoCodeSearchQuery,
    search_tokens: &mut Vec<String>,
    token: &str,
) {
    match classify_repo_code_search_token(token) {
        RepoCodeSearchToken::Language(value) => apply_language_filter(spec, value),
        RepoCodeSearchToken::Repo(value) => apply_repo_filter(spec, value),
        RepoCodeSearchToken::Kind(value) => apply_kind_filter(spec, value),
        RepoCodeSearchToken::AstPattern(value) => apply_ast_pattern(spec, value),
        RepoCodeSearchToken::Search(value) => search_tokens.push(value.to_string()),
    }
}

fn classify_repo_code_search_token(token: &str) -> RepoCodeSearchToken<'_> {
    if let Some(value) = token.strip_prefix("lang:") {
        return RepoCodeSearchToken::Language(value);
    }

    if let Some(value) = token.strip_prefix("repo:") {
        return RepoCodeSearchToken::Repo(value);
    }

    if let Some(value) = valid_kind_filter_token(token) {
        return RepoCodeSearchToken::Kind(value);
    }

    if let Some(value) = ast_pattern_token(token) {
        return RepoCodeSearchToken::AstPattern(value);
    }

    RepoCodeSearchToken::Search(token)
}

fn valid_kind_filter_token(token: &str) -> Option<&str> {
    let value = token.strip_prefix("kind:")?;
    let normalized = value.trim().to_ascii_lowercase();
    REPO_CODE_SEARCH_KIND_FILTER_VALUES
        .contains(&normalized.as_str())
        .then_some(value)
}

fn ast_pattern_token(token: &str) -> Option<&str> {
    token
        .strip_prefix("ast:")
        .or_else(|| token.strip_prefix("sg:"))
}

fn apply_language_filter(spec: &mut ParsedRepoCodeSearchQuery, value: &str) {
    let normalized = value.trim().to_ascii_lowercase();
    if !normalized.is_empty() {
        spec.language_filters.insert(normalized);
    }
}

fn apply_repo_filter(spec: &mut ParsedRepoCodeSearchQuery, value: &str) {
    let repo_id = value.trim();
    if !repo_id.is_empty() {
        spec.repo = Some(repo_id.to_string());
    }
}

fn apply_kind_filter(spec: &mut ParsedRepoCodeSearchQuery, value: &str) {
    let normalized = value.trim().to_ascii_lowercase();
    spec.kind_filters.insert(normalized);
}

fn apply_ast_pattern(spec: &mut ParsedRepoCodeSearchQuery, value: &str) {
    let normalized = strip_matching_quotes(value.trim());
    if !normalized.is_empty() {
        spec.ast_pattern = Some(normalized);
    }
}

fn search_term_from_tokens(search_tokens: &[String]) -> Option<String> {
    (!search_tokens.is_empty()).then(|| search_tokens.join(" "))
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
