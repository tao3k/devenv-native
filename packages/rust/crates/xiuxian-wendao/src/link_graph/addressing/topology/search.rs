//! Topology index search and fuzzy path resolution.

use crate::link_graph::addressing::topology::helpers::{path_match_suffix, similarity_ratio};
use crate::link_graph::addressing::topology::{MatchType, PathEntry, PathMatch, TopologyIndex};
use crate::search::{FuzzyMatcher, FuzzySearchOptions, LexicalMatcher};

impl TopologyIndex {
    /// Find a node by exact structural path within a document.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn exact_path(&self, doc_id: &str, components: &[String]) -> Option<&PathEntry> {
        let entries = self.by_doc.get(doc_id)?;
        entries.iter().find(|e| e.path.as_slice() == components)
    }

    /// Find a node by exact or case-insensitive path.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn path_case_insensitive(&self, doc_id: &str, components: &[String]) -> Option<PathMatch> {
        // Try exact match first
        if let Some(entry) = self.exact_path(doc_id, components) {
            return Some(PathMatch {
                doc_id: doc_id.to_string(),
                path: entry.path.clone(),
                similarity_score: 1.0,
                entry: entry.clone(),
                match_type: MatchType::Exact,
            });
        }

        // Try case-insensitive match
        let entries = self.by_doc.get(doc_id)?;
        let lower_components: Vec<String> = components.iter().map(|c| c.to_lowercase()).collect();

        for entry in entries {
            let entry_lower: Vec<String> = entry.path.iter().map(|p| p.to_lowercase()).collect();
            if entry_lower == lower_components {
                return Some(PathMatch {
                    doc_id: doc_id.to_string(),
                    path: entry.path.clone(),
                    similarity_score: 0.95,
                    entry: entry.clone(),
                    match_type: MatchType::CaseInsensitive,
                });
            }
        }

        None
    }

    /// Find a node by content hash (self-healing).
    #[must_use]
    pub fn find_by_hash(&self, hash: &str) -> Option<&PathEntry> {
        self.hash_index.get(hash)
    }

    /// Fuzzy path matching with path drift tolerance.
    ///
    /// Returns matches sorted by similarity score (highest first).
    #[must_use]
    pub fn fuzzy_resolve(&self, query: &str, max_results: usize) -> Vec<PathMatch> {
        self.fuzzy_resolve_with_options(query, max_results, FuzzySearchOptions::path_search())
    }

    /// Fuzzy path matching with explicit fuzzy options.
    ///
    /// # Panics
    ///
    /// Panics if the lexical matcher unexpectedly returns an error. The current
    /// in-memory matcher implementation is designed to be infallible.
    #[must_use]
    pub fn fuzzy_resolve_with_options(
        &self,
        query: &str,
        max_results: usize,
        options: FuzzySearchOptions,
    ) -> Vec<PathMatch> {
        let query_lower = query.to_lowercase();
        let mut matches = self.exact_title_matches(&query_lower);
        self.extend_suffix_matches(&query_lower, &mut matches);
        self.extend_title_substring_matches(&query_lower, &mut matches);
        if matches.is_empty() {
            matches = self.lexical_title_fuzzy_matches(query, max_results, options);
        }

        sorted_limited_matches(matches, max_results)
    }

    /// Get all path entries for a document.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn entries_for_doc(&self, doc_id: &str) -> Option<&Vec<PathEntry>> {
        self.by_doc.get(doc_id)
    }

    /// Get the total number of indexed entries across all documents.
    #[must_use]
    pub fn total_entries(&self) -> usize {
        self.by_doc.values().map(std::vec::Vec::len).sum()
    }

    /// Get all document IDs in the index.
    #[must_use]
    pub fn doc_ids(&self) -> Vec<&str> {
        self.by_doc
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Find a path entry by its `node_id` (Blueprint Section 2.2 skeleton validation).
    ///
    /// This is used for skeleton re-ranking to validate vector search results
    /// against the current AST structure.
    #[must_use]
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn find_by_node_id(&self, node_id: &str) -> Option<&PathEntry> {
        self.hash_index
            .values()
            .find(|entry| entry.node_id == node_id)
    }

    fn exact_title_matches(&self, query_lower: &str) -> Vec<PathMatch> {
        self.title_index
            .get(query_lower)
            .into_iter()
            .flatten()
            .map(exact_title_match)
            .collect()
    }

    fn extend_suffix_matches(&self, query_lower: &str, matches: &mut Vec<PathMatch>) {
        for entry in self.all_entries() {
            if suffix_matches_query(entry, query_lower) && !contains_entry_match(matches, entry) {
                matches.push(suffix_path_match(entry));
            }
        }
    }

    fn extend_title_substring_matches(&self, query_lower: &str, matches: &mut Vec<PathMatch>) {
        for (title, title_matches) in &self.title_index {
            if title.contains(query_lower) && title != query_lower {
                extend_missing_title_substring_matches(matches, query_lower, title, title_matches);
            }
        }
    }

    fn lexical_title_fuzzy_matches(
        &self,
        query: &str,
        max_results: usize,
        options: FuzzySearchOptions,
    ) -> Vec<PathMatch> {
        fn path_entry_title(entry: &PathEntry) -> &str {
            entry.title.as_str()
        }

        let candidates = self.all_entries().cloned().collect::<Vec<_>>();
        let lexical_matcher = LexicalMatcher::new(candidates.as_slice(), path_entry_title, options);
        lexical_matcher
            .search(query, max_results)
            .expect("lexical matcher is infallible")
            .into_iter()
            .map(|fuzzy_match| title_fuzzy_match(fuzzy_match.item, fuzzy_match.score))
            .collect()
    }

    fn all_entries(&self) -> impl Iterator<Item = &PathEntry> {
        self.by_doc.values().flat_map(|entries| entries.iter())
    }
}

fn exact_title_match(match_: &PathMatch) -> PathMatch {
    let mut scored = match_.clone();
    scored.similarity_score = 1.0;
    scored.match_type = MatchType::Exact;
    scored
}

fn suffix_matches_query(entry: &PathEntry, query_lower: &str) -> bool {
    let path_lower: Vec<String> = entry.path.iter().map(|part| part.to_lowercase()).collect();
    path_match_suffix(&path_lower, query_lower)
}

fn suffix_path_match(entry: &PathEntry) -> PathMatch {
    PathMatch {
        doc_id: entry.doc_id.clone(),
        path: entry.path.clone(),
        similarity_score: 0.85,
        entry: entry.clone(),
        match_type: MatchType::Suffix,
    }
}

fn extend_missing_title_substring_matches(
    matches: &mut Vec<PathMatch>,
    query_lower: &str,
    title: &str,
    title_matches: &[PathMatch],
) {
    let missing = title_matches
        .iter()
        .filter(|match_| !contains_path_match(matches, match_))
        .map(|match_| title_substring_match(match_, query_lower, title))
        .collect::<Vec<_>>();
    matches.extend(missing);
}

fn title_substring_match(match_: &PathMatch, query_lower: &str, title: &str) -> PathMatch {
    let mut scored = match_.clone();
    scored.similarity_score = 0.7 + similarity_ratio(query_lower.len(), title.len()).min(0.25);
    scored.match_type = MatchType::TitleSubstring;
    scored
}

fn title_fuzzy_match(entry: PathEntry, score: f32) -> PathMatch {
    PathMatch {
        doc_id: entry.doc_id.clone(),
        path: entry.path.clone(),
        similarity_score: score,
        entry,
        match_type: MatchType::TitleFuzzy,
    }
}

fn contains_entry_match(matches: &[PathMatch], entry: &PathEntry) -> bool {
    matches
        .iter()
        .any(|existing| existing.entry.node_id == entry.node_id && existing.doc_id == entry.doc_id)
}

fn contains_path_match(matches: &[PathMatch], match_: &PathMatch) -> bool {
    matches.iter().any(|existing| {
        existing.entry.node_id == match_.entry.node_id && existing.doc_id == match_.doc_id
    })
}

fn sorted_limited_matches(mut matches: Vec<PathMatch>, max_results: usize) -> Vec<PathMatch> {
    matches.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(max_results);
    matches
}
