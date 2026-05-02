//! Generic search-family route contracts and shared metadata validation.

/// Canonical generic search query text metadata header for Wendao Flight requests.
pub const WENDAO_SEARCH_QUERY_HEADER: &str = "x-wendao-search-query";
/// Canonical generic search result-limit metadata header for Wendao Flight requests.
pub const WENDAO_SEARCH_LIMIT_HEADER: &str = "x-wendao-search-limit";
/// Canonical generic search intent metadata header for Wendao Flight requests.
pub const WENDAO_SEARCH_INTENT_HEADER: &str = "x-wendao-search-intent";
/// Canonical generic search repository hint metadata header for Wendao Flight requests.
pub const WENDAO_SEARCH_REPO_HEADER: &str = "x-wendao-search-repo";
/// Stable route for the search-intent contract.
pub const SEARCH_INTENT_ROUTE: &str = "/search/intent";
/// Stable route for the general knowledge-search contract.
pub const SEARCH_KNOWLEDGE_ROUTE: &str = "/search/knowledge";
/// Stable route for the search-references contract.
pub const SEARCH_REFERENCES_ROUTE: &str = "/search/references";
/// Stable route for the search-symbols contract.
pub const SEARCH_SYMBOLS_ROUTE: &str = "/search/symbols";
