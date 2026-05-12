//! Lightweight query intent extractor.
//!
//! Decomposes a natural-language query into structured intent signals:
//! - **action**: the verb / desired operation (e.g. "commit", "search", "create")
//! - **target**: the object / domain the action applies to (e.g. "code", "file", "git")
//! - **context**: additional qualifiers / modifiers (e.g. "python", "async", "memory")
//! - **keywords**: all significant tokens extracted from the query
//!
//! These signals feed into the routing + fusion pipeline so that KG-rerank
//! and dynamic weight selection can work at the semantic level rather than
//! relying on raw substring matching alone.

// -----------------------------------------------------------------------
// Action vocabulary — maps natural-language verbs to canonical actions.
// -----------------------------------------------------------------------

pub(super) const ACTION_VERBS: &[(&str, &str)] = &[
    // Search / Retrieval
    ("search", "search"),
    ("find", "search"),
    ("look", "search"),
    ("lookup", "search"),
    ("query", "search"),
    ("recall", "search"),
    ("retrieve", "search"),
    ("fetch", "search"),
    ("get", "search"),
    ("locate", "search"),
    // Create / Write
    ("create", "create"),
    ("make", "create"),
    ("add", "create"),
    ("write", "create"),
    ("generate", "create"),
    ("build", "create"),
    ("scaffold", "create"),
    ("init", "create"),
    ("initialize", "create"),
    ("new", "create"),
    // Modify / Update
    ("update", "update"),
    ("edit", "update"),
    ("modify", "update"),
    ("change", "update"),
    ("refactor", "update"),
    ("rename", "update"),
    ("fix", "update"),
    ("patch", "update"),
    // Delete / Remove
    ("delete", "delete"),
    ("remove", "delete"),
    ("drop", "delete"),
    ("clean", "delete"),
    ("purge", "delete"),
    // Git operations
    ("commit", "commit"),
    ("push", "push"),
    ("pull", "pull"),
    ("merge", "merge"),
    ("rebase", "rebase"),
    ("branch", "branch"),
    ("checkout", "checkout"),
    ("diff", "diff"),
    ("status", "status"),
    ("log", "log"),
    ("stash", "stash"),
    // Run / Execute
    ("run", "run"),
    ("execute", "run"),
    ("start", "run"),
    ("launch", "run"),
    ("test", "test"),
    ("lint", "lint"),
    ("format", "format"),
    // Analyze / Inspect
    ("analyze", "analyze"),
    ("inspect", "analyze"),
    ("explain", "analyze"),
    ("describe", "analyze"),
    ("show", "analyze"),
    ("list", "list"),
    ("count", "count"),
    // Index / Sync
    ("index", "index"),
    ("reindex", "index"),
    ("sync", "sync"),
    ("embed", "embed"),
    // Research
    ("research", "research"),
    ("crawl", "crawl"),
    ("browse", "crawl"),
];

// -----------------------------------------------------------------------
// Domain vocabulary — maps nouns/domains to canonical targets.
// -----------------------------------------------------------------------

pub(super) const DOMAIN_TARGETS: &[(&str, &str)] = &[
    // Source control
    ("git", "git"),
    ("repo", "git"),
    ("repository", "git"),
    ("branch", "git"),
    ("commit", "git"),
    // Knowledge
    ("knowledge", "knowledge"),
    ("memory", "memory"),
    ("note", "knowledge"),
    ("notes", "knowledge"),
    ("graph", "knowledge"),
    // Code
    ("code", "code"),
    ("function", "code"),
    ("class", "code"),
    ("module", "code"),
    ("file", "code"),
    ("files", "code"),
    ("codebase", "code"),
    ("source", "code"),
    // Skills / Tools
    ("skill", "skill"),
    ("skills", "skill"),
    ("tool", "skill"),
    ("tools", "skill"),
    ("command", "skill"),
    ("commands", "skill"),
    // Web / Research
    ("web", "web"),
    ("internet", "web"),
    ("url", "web"),
    ("page", "web"),
    ("site", "web"),
    ("website", "web"),
    // Database
    ("database", "database"),
    ("db", "database"),
    ("table", "database"),
    ("vector", "database"),
    ("index", "database"),
    // Documentation
    ("doc", "docs"),
    ("docs", "docs"),
    ("documentation", "docs"),
    ("readme", "docs"),
    ("spec", "docs"),
    ("reference", "docs"),
    // Tests
    ("test", "test"),
    ("tests", "test"),
    ("testing", "test"),
];

/// English stop words (common function words to filter out).
pub(super) const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "shall", "should", "may", "might", "must", "can",
    "could", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "through",
    "during", "before", "after", "above", "below", "between", "out", "off", "over", "under",
    "again", "further", "then", "once", "here", "there", "when", "where", "why", "how", "all",
    "both", "each", "every", "few", "more", "most", "other", "some", "such", "no", "nor", "not",
    "only", "own", "same", "so", "than", "too", "very", "just", "also", "about", "up", "down",
    "if", "or", "and", "but", "because", "until", "while", "it", "its", "this", "that", "these",
    "those", "my", "your", "his", "her", "our", "their", "what", "which", "who", "whom", "me",
    "him", "them", "i", "you", "he", "she", "we", "they", "please", "want", "need", "help", "like",
    "using",
];
