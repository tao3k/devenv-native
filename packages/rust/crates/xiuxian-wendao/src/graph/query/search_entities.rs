use crate::entity::Entity;
use crate::graph::{KnowledgeGraph, read_lock};
use std::collections::HashMap;
use std::collections::HashSet;

/// Scoring weights for entity search relevance.
const EXACT_NAME_SCORE: f64 = 1.0;
const ALIAS_EXACT_SCORE: f64 = 0.95;
const TOKEN_FULL_OVERLAP_SCORE: f64 = 0.85;
const SUBSTRING_NAME_SCORE: f64 = 0.7;
const ALIAS_SUBSTRING_SCORE: f64 = 0.65;
const TOKEN_PARTIAL_OVERLAP_SCORE: f64 = 0.5;
const DESCRIPTION_MATCH_SCORE: f64 = 0.3;
const FUZZY_MATCH_THRESHOLD: f32 = 0.75;
const FUZZY_MATCH_SCORE: f64 = 0.4;

impl KnowledgeGraph {
    /// Search entities with multi-signal relevance scoring.
    ///
    /// Scoring signals (in priority order):
    /// 1. Exact name match (1.0)
    /// 2. Exact alias match (0.95)
    /// 3. Full token overlap — all query tokens appear in name tokens (0.85)
    /// 4. Name substring match (0.7)
    /// 5. Alias substring match (0.65)
    /// 6. Partial token overlap — some query tokens match name tokens (0.5)
    /// 7. Fuzzy name match — Levenshtein similarity ≥ 0.75 (0.4)
    /// 8. Description substring match (0.3)
    #[must_use]
    pub fn search_entities(&self, query: &str, limit: i32) -> Vec<Entity> {
        let query = EntitySearchQuery::new(query);
        if query.is_empty() {
            return Vec::new();
        }

        let entities = read_lock::<HashMap<String, Entity>>(&self.entities);
        let mut scored = score_search_entities(entities.values(), &query);
        rank_and_limit_entities(&mut scored, limit)
    }
}

struct EntitySearchQuery {
    lower: String,
    tokens: Vec<String>,
}

impl EntitySearchQuery {
    fn new(query: &str) -> Self {
        let lower = query.to_lowercase();
        let tokens = tokenize_entity_query(lower.as_str());
        Self { lower, tokens }
    }

    fn is_empty(&self) -> bool {
        self.lower.is_empty()
    }
}

fn tokenize_entity_query(query_lower: &str) -> Vec<String> {
    query_lower
        .split(is_entity_query_separator)
        .filter(|token| !token.is_empty() && token.len() >= 2)
        .map(str::to_string)
        .collect()
}

fn is_entity_query_separator(candidate: char) -> bool {
    candidate.is_whitespace() || matches!(candidate, '.' | '_' | '-')
}

fn score_search_entities<'a>(
    entities: impl Iterator<Item = &'a Entity>,
    query: &EntitySearchQuery,
) -> Vec<(f64, Entity)> {
    entities
        .filter_map(|entity| score_search_entity(entity, query))
        .collect()
}

fn score_search_entity(entity: &Entity, query: &EntitySearchQuery) -> Option<(f64, Entity)> {
    let name_lower = entity.name.to_lowercase();
    let best_score = best_entity_search_score(entity, name_lower.as_str(), query);
    (best_score > 0.0).then(|| {
        let final_score = best_score * (0.8 + 0.2 * f64::from(entity.confidence));
        (final_score, entity.clone())
    })
}

fn best_entity_search_score(entity: &Entity, name_lower: &str, query: &EntitySearchQuery) -> f64 {
    [
        exact_name_score(name_lower, query),
        exact_alias_score(entity, query),
        token_overlap_score(name_lower, query),
        name_substring_score(name_lower, query),
        alias_substring_score(entity, query),
        fuzzy_name_score(name_lower, query),
        description_match_score(entity, query),
    ]
    .into_iter()
    .fold(0.0, f64::max)
}

fn exact_name_score(name_lower: &str, query: &EntitySearchQuery) -> f64 {
    if name_lower == query.lower {
        EXACT_NAME_SCORE
    } else {
        0.0
    }
}

fn exact_alias_score(entity: &Entity, query: &EntitySearchQuery) -> f64 {
    if entity
        .aliases
        .iter()
        .any(|alias| alias.to_lowercase() == query.lower)
    {
        ALIAS_EXACT_SCORE
    } else {
        0.0
    }
}

fn token_overlap_score(name_lower: &str, query: &EntitySearchQuery) -> f64 {
    if query.tokens.is_empty() {
        return 0.0;
    }

    let name_tokens: HashSet<&str> = name_lower
        .split(is_entity_query_separator)
        .filter(|token| !token.is_empty() && token.len() >= 2)
        .collect();
    if name_tokens.is_empty() {
        return 0.0;
    }

    let matched = query
        .tokens
        .iter()
        .filter(|query_token| token_matches_name_tokens(query_token.as_str(), &name_tokens))
        .count();
    token_overlap_score_from_match_count(matched, query.tokens.len())
}

fn token_matches_name_tokens(query_token: &str, name_tokens: &HashSet<&str>) -> bool {
    name_tokens
        .iter()
        .any(|name_token| name_token.contains(query_token) || query_token.contains(name_token))
}

fn token_overlap_score_from_match_count(matched: usize, token_count: usize) -> f64 {
    if matched == token_count && matched > 0 {
        return TOKEN_FULL_OVERLAP_SCORE;
    }
    if matched == 0 {
        return 0.0;
    }
    let matched_u32 = u32::try_from(matched).unwrap_or(u32::MAX);
    let token_count_u32 = u32::try_from(token_count).unwrap_or(u32::MAX);
    TOKEN_PARTIAL_OVERLAP_SCORE * (f64::from(matched_u32) / f64::from(token_count_u32))
}

fn name_substring_score(name_lower: &str, query: &EntitySearchQuery) -> f64 {
    if name_lower.contains(query.lower.as_str()) || query.lower.contains(name_lower) {
        SUBSTRING_NAME_SCORE
    } else {
        0.0
    }
}

fn alias_substring_score(entity: &Entity, query: &EntitySearchQuery) -> f64 {
    if entity.aliases.iter().any(|alias| {
        let alias_lower = alias.to_lowercase();
        alias_lower.contains(query.lower.as_str()) || query.lower.contains(alias_lower.as_str())
    }) {
        ALIAS_SUBSTRING_SCORE
    } else {
        0.0
    }
}

fn fuzzy_name_score(name_lower: &str, query: &EntitySearchQuery) -> f64 {
    let similarity = KnowledgeGraph::name_similarity(query.lower.as_str(), name_lower);
    if similarity >= FUZZY_MATCH_THRESHOLD {
        FUZZY_MATCH_SCORE * f64::from(similarity)
    } else {
        0.0
    }
}

fn description_match_score(entity: &Entity, query: &EntitySearchQuery) -> f64 {
    if entity
        .description
        .to_lowercase()
        .contains(query.lower.as_str())
    {
        DESCRIPTION_MATCH_SCORE
    } else {
        0.0
    }
}

fn rank_and_limit_entities(scored: &mut Vec<(f64, Entity)>, limit: i32) -> Vec<Entity> {
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let bounded_limit = usize::try_from(limit).unwrap_or(0);
    scored.truncate(bounded_limit);
    scored.drain(..).map(|(_, entity)| entity).collect()
}
