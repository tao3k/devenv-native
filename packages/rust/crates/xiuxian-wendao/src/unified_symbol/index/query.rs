use std::collections::HashSet;

use crate::search::{FuzzyMatcher, FuzzySearchOptions, LexicalMatcher, SearchDocument};
use crate::unified_symbol::symbol::SymbolSource;
use crate::unified_symbol::{UnifiedSymbol, UnifiedSymbolIndex};

impl UnifiedSymbolIndex {
    /// Search for project symbols.
    #[must_use]
    pub fn search_project(&self, query: &str, limit: usize) -> Vec<UnifiedSymbol> {
        self.search_project_with_options(query, limit, FuzzySearchOptions::symbol_search())
    }

    /// Search for project symbols with explicit fuzzy options.
    #[must_use]
    pub fn search_project_with_options(
        &self,
        query: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        self.search_with_filter(query, limit, Some("project"), options)
    }

    /// Search for external symbols.
    #[must_use]
    pub fn search_external(&self, query: &str, limit: usize) -> Vec<UnifiedSymbol> {
        self.search_external_with_options(query, limit, FuzzySearchOptions::symbol_search())
    }

    /// Search for external symbols with explicit fuzzy options.
    #[must_use]
    pub fn search_external_with_options(
        &self,
        query: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        self.search_with_filter(query, limit, Some("external"), options)
    }

    /// Search for symbols within a specific crate.
    #[must_use]
    pub fn search_crate(&self, crate_name: &str, query: &str, limit: usize) -> Vec<UnifiedSymbol> {
        self.search_crate_with_options(
            crate_name,
            query,
            limit,
            FuzzySearchOptions::symbol_search(),
        )
    }

    /// Search for symbols within a specific crate with explicit fuzzy options.
    #[must_use]
    pub fn search_crate_with_options(
        &self,
        crate_name: &str,
        query: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        let results = self.search_unified_with_options(query, limit * 2, options);
        results
            .into_iter()
            .filter(|s| s.crate_name == crate_name)
            .take(limit)
            .collect()
    }

    /// Search across both project and external symbols.
    #[must_use]
    pub fn search_unified(&self, query_str: &str, limit: usize) -> Vec<UnifiedSymbol> {
        self.search_unified_with_options(query_str, limit, FuzzySearchOptions::symbol_search())
    }

    /// Search across both project and external symbols with explicit fuzzy options.
    #[must_use]
    pub fn search_unified_with_options(
        &self,
        query_str: &str,
        limit: usize,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        self.search_with_filter(query_str, limit, None, options)
    }

    fn search_with_filter(
        &self,
        query_str: &str,
        limit: usize,
        source_filter: Option<&str>,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        if let Some(exact_results) = self.search_tantivy_exact(query_str, limit, source_filter) {
            push_unique_symbols(&mut results, &mut seen, exact_results, limit);
        }

        if results.len() < limit
            && let Some(fuzzy_results) =
                self.search_tantivy_fuzzy(query_str, limit, source_filter, options)
        {
            push_unique_symbols(&mut results, &mut seen, fuzzy_results, limit);
        }

        if results.len() < limit {
            let fallback_results =
                self.search_memory_fallback(query_str, limit, source_filter, options);
            push_unique_symbols(&mut results, &mut seen, fallback_results, limit);
        }

        results
    }

    fn search_tantivy_exact(
        &self,
        query_str: &str,
        limit: usize,
        source_filter: Option<&str>,
    ) -> Option<Vec<UnifiedSymbol>> {
        let Ok(records) = self
            .search_index
            .search_exact(query_str, limit.saturating_mul(2))
        else {
            return None;
        };
        Some(self.symbols_from_search_documents(records, limit, source_filter))
    }

    fn search_tantivy_fuzzy(
        &self,
        query_str: &str,
        limit: usize,
        source_filter: Option<&str>,
        options: FuzzySearchOptions,
    ) -> Option<Vec<UnifiedSymbol>> {
        let Ok(matches) =
            self.search_index
                .search_fuzzy(query_str, limit.saturating_mul(2), options)
        else {
            return None;
        };

        Some(
            matches
                .into_iter()
                .filter_map(|fuzzy_match| {
                    self.symbol_from_search_id(fuzzy_match.item.id.as_str(), source_filter)
                })
                .take(limit)
                .collect(),
        )
    }

    fn search_memory_fallback(
        &self,
        query_str: &str,
        limit: usize,
        source_filter: Option<&str>,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        let query_lower = query_str.to_lowercase();
        let mut results = self.exact_memory_symbols(query_lower.as_str(), source_filter);
        if results.len() < limit {
            results.extend(self.prefix_memory_symbols(
                query_lower.as_str(),
                source_filter,
                limit - results.len(),
            ));
        }

        if !results.is_empty() {
            results.truncate(limit);
            return results;
        }

        self.fuzzy_memory_symbols(query_str, limit, source_filter, options)
    }

    fn exact_memory_symbols(
        &self,
        query_lower: &str,
        source_filter: Option<&str>,
    ) -> Vec<UnifiedSymbol> {
        self.by_name
            .get(query_lower)
            .into_iter()
            .flatten()
            .filter_map(|&idx| self.symbols.get(idx))
            .filter(|symbol| Self::matches_filter(symbol, source_filter))
            .cloned()
            .collect()
    }

    fn prefix_memory_symbols(
        &self,
        query_lower: &str,
        source_filter: Option<&str>,
        limit: usize,
    ) -> Vec<UnifiedSymbol> {
        self.by_name
            .iter()
            .filter(|(name, _)| name.as_str() != query_lower && name.starts_with(query_lower))
            .flat_map(|(_, indices)| indices.iter())
            .filter_map(|&idx| self.symbols.get(idx))
            .filter(|symbol| Self::matches_filter(symbol, source_filter))
            .take(limit)
            .cloned()
            .collect()
    }

    fn fuzzy_memory_symbols(
        &self,
        query_str: &str,
        limit: usize,
        source_filter: Option<&str>,
        options: FuzzySearchOptions,
    ) -> Vec<UnifiedSymbol> {
        let filtered_symbols = self
            .symbols
            .iter()
            .filter(|symbol| Self::matches_filter(symbol, source_filter))
            .cloned()
            .collect::<Vec<_>>();
        let matcher =
            LexicalMatcher::new(filtered_symbols.as_slice(), unified_symbol_name, options);
        let fuzzy_matches = matcher
            .search(query_str, limit)
            .expect("lexical matcher is infallible");
        fuzzy_matches
            .into_iter()
            .map(|fuzzy_match| fuzzy_match.item)
            .take(limit)
            .collect()
    }

    fn matches_filter(symbol: &UnifiedSymbol, filter: Option<&str>) -> bool {
        match filter {
            None => true,
            Some("project") => symbol.source == SymbolSource::Project,
            Some("external") => matches!(symbol.source, SymbolSource::External(_)),
            _ => false,
        }
    }

    /// Find all external symbol usages for a given crate.
    #[must_use]
    pub fn find_external_usage(&self, crate_name: &str) -> Vec<String> {
        self.external_usage
            .get(crate_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Find all symbols defined in a given project file.
    #[must_use]
    pub fn find_symbols_in_file(&self, file_path: &str) -> Vec<String> {
        self.project_files
            .get(file_path)
            .cloned()
            .unwrap_or_default()
    }

    fn symbols_from_search_documents(
        &self,
        records: Vec<SearchDocument>,
        limit: usize,
        source_filter: Option<&str>,
    ) -> Vec<UnifiedSymbol> {
        records
            .into_iter()
            .filter_map(|record| self.symbol_from_search_id(record.id.as_str(), source_filter))
            .take(limit)
            .collect()
    }

    fn symbol_from_search_id(
        &self,
        search_id: &str,
        source_filter: Option<&str>,
    ) -> Option<UnifiedSymbol> {
        let idx = search_id.parse::<usize>().ok()?;
        let symbol = self.symbols.get(idx)?;
        Self::matches_filter(symbol, source_filter).then(|| symbol.clone())
    }
}

fn unified_symbol_name(symbol: &UnifiedSymbol) -> &str {
    symbol.name.as_str()
}

fn push_unique_symbols(
    results: &mut Vec<UnifiedSymbol>,
    seen: &mut HashSet<String>,
    candidates: Vec<UnifiedSymbol>,
    limit: usize,
) {
    for symbol in candidates {
        let dedupe_key = symbol_dedupe_key(&symbol);
        if seen.insert(dedupe_key) {
            results.push(symbol);
        }
        if results.len() >= limit {
            break;
        }
    }
}

fn symbol_dedupe_key(symbol: &UnifiedSymbol) -> String {
    format!(
        "{}::{}::{}::{}::{:?}",
        symbol.name, symbol.kind, symbol.location, symbol.crate_name, symbol.source
    )
}
