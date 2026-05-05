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
        let entities = read_lock::<HashMap<String, Entity>>(&self.entities);
        let query_lower = query.to_lowercase();

        if query_lower.is_empty() {
            return Vec::new();
        }

        // Tokenize query: split on whitespace, dots, underscores, hyphens.
        let query_tokens: Vec<&str> = query_lower
            .split(|c: char| c.is_whitespace() || c == '.' || c == '_' || c == '-')
            .filter(|t: &&str| !t.is_empty() && t.len() >= 2)
            .collect();

        let mut scored: Vec<(f64, Entity)> = Vec::new();

        for entity in entities.values() {
            let name_lower = entity.name.to_lowercase();
            let mut best_score: f64 = 0.0;

            // Signal 1: Exact name match.
            if name_lower == query_lower {
                best_score = EXACT_NAME_SCORE;
            }

            // Signal 2: Exact alias match.
            if best_score < ALIAS_EXACT_SCORE {
                for alias in &entity.aliases {
                    let alias: &String = alias;
                    if alias.to_lowercase() == query_lower {
                        best_score = best_score.max(ALIAS_EXACT_SCORE);
                        break;
                    }
                }
            }

            // Signal 3/6: Token overlap scoring.
            if best_score < TOKEN_FULL_OVERLAP_SCORE && !query_tokens.is_empty() {
                let name_tokens: HashSet<&str> = name_lower
                    .split(|c: char| c.is_whitespace() || c == '.' || c == '_' || c == '-')
                    .filter(|t: &&str| !t.is_empty() && t.len() >= 2)
                    .collect();

                if !name_tokens.is_empty() {
                    let matched = query_tokens
                        .iter()
                        .filter(|qt: &&&str| {
                            name_tokens
                                .iter()
                                .any(|nt: &&str| nt.contains(**qt) || qt.contains(nt))
                        })
                        .count();

                    if matched == query_tokens.len() && matched > 0 {
                        // All query tokens matched.
                        best_score = best_score.max(TOKEN_FULL_OVERLAP_SCORE);
                    } else if matched > 0 {
                        // Partial: scale between 0.3 and TOKEN_PARTIAL_OVERLAP_SCORE.
                        let matched_u32 = u32::try_from(matched).unwrap_or(u32::MAX);
                        let token_count_u32 = u32::try_from(query_tokens.len()).unwrap_or(u32::MAX);
                        let ratio = f64::from(matched_u32) / f64::from(token_count_u32);
                        let partial = TOKEN_PARTIAL_OVERLAP_SCORE * ratio;
                        best_score = best_score.max(partial);
                    }
                }
            }

            // Signal 4: Name substring match.
            if best_score < SUBSTRING_NAME_SCORE
                && (name_lower.contains(&query_lower) || query_lower.contains(&name_lower))
            {
                best_score = best_score.max(SUBSTRING_NAME_SCORE);
            }

            // Signal 5: Alias substring match.
            if best_score < ALIAS_SUBSTRING_SCORE {
                for alias in &entity.aliases {
                    let alias: &String = alias;
                    let alias_lower = alias.to_lowercase();
                    if alias_lower.contains(&query_lower) || query_lower.contains(&alias_lower) {
                        best_score = best_score.max(ALIAS_SUBSTRING_SCORE);
                        break;
                    }
                }
            }

            // Signal 7: Fuzzy name match (Levenshtein).
            if best_score < FUZZY_MATCH_SCORE {
                let sim = KnowledgeGraph::name_similarity(&query_lower, &name_lower);
                if sim >= FUZZY_MATCH_THRESHOLD {
                    best_score = best_score.max(FUZZY_MATCH_SCORE * f64::from(sim));
                }
            }

            // Signal 8: Description substring match.
            if best_score < DESCRIPTION_MATCH_SCORE {
                let desc_lower = entity.description.to_lowercase();
                if desc_lower.contains(&query_lower) {
                    best_score = best_score.max(DESCRIPTION_MATCH_SCORE);
                }
            }

            if best_score > 0.0 {
                // Confidence boost: entities with higher confidence rank higher.
                let final_score = best_score * (0.8 + 0.2 * f64::from(entity.confidence));
                scored.push((final_score, entity.clone()));
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let bounded_limit = usize::try_from(limit).unwrap_or(0);
        scored.truncate(bounded_limit);
        scored.into_iter().map(|(_, e)| e).collect()
    }
}
