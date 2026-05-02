use crate::entity::{Entity, EntityType, Relation, RelationType};
use crate::graph::{KnowledgeGraph, read_lock};
use std::collections::{HashMap, HashSet};

struct ToolRelevanceContext<'a> {
    entities_by_name: &'a HashMap<String, String>,
    entities: &'a HashMap<String, Entity>,
    outgoing: &'a HashMap<String, HashSet<String>>,
    incoming: &'a HashMap<String, HashSet<String>>,
    relations: &'a HashMap<String, Relation>,
}

fn push_seed_if_absent(seed_entities: &mut Vec<(String, f64)>, entity_name: &str, score: f64) {
    if seed_entities.iter().any(|(name, _)| name == entity_name) {
        return;
    }
    seed_entities.push((entity_name.to_string(), score));
}

fn add_exact_and_keyword_matches(
    term_lower: &str,
    context: &ToolRelevanceContext<'_>,
    seed_entities: &mut Vec<(String, f64)>,
) {
    if context.entities_by_name.contains_key(term_lower) {
        push_seed_if_absent(seed_entities, term_lower, 1.0);
    }
    let keyword_name = format!("keyword:{term_lower}");
    if context.entities_by_name.contains_key(&keyword_name) {
        push_seed_if_absent(seed_entities, &keyword_name, 0.8);
    }
}

fn add_alias_matches(
    term_lower: &str,
    context: &ToolRelevanceContext<'_>,
    seed_entities: &mut Vec<(String, f64)>,
) {
    for entity in context.entities.values() {
        for alias in &entity.aliases {
            if alias.eq_ignore_ascii_case(term_lower) {
                push_seed_if_absent(seed_entities, &entity.name, 0.85);
                break;
            }
        }
    }
}

fn add_substring_matches(
    term_lower: &str,
    context: &ToolRelevanceContext<'_>,
    seed_entities: &mut Vec<(String, f64)>,
) {
    let keyword_name = format!("keyword:{term_lower}");
    for name in context.entities_by_name.keys() {
        if name.contains(term_lower) && name != term_lower && name != &keyword_name {
            push_seed_if_absent(seed_entities, name, 0.5);
        }
    }
}

fn add_token_overlap_matches(
    term_lower: &str,
    context: &ToolRelevanceContext<'_>,
    seed_entities: &mut Vec<(String, f64)>,
) {
    for name in context.entities_by_name.keys() {
        let has_token_overlap = name
            .split(['.', '_', '-'])
            .filter(|token| !token.is_empty())
            .any(|token| token == term_lower || token.contains(term_lower));
        if has_token_overlap {
            push_seed_if_absent(seed_entities, name, 0.4);
        }
    }
}

fn collect_seed_entities(
    query_terms: &[String],
    context: &ToolRelevanceContext<'_>,
) -> Vec<(String, f64)> {
    let mut seed_entities = Vec::new();
    for term in query_terms {
        let term_lower = term.to_lowercase();
        if term_lower.is_empty() {
            continue;
        }
        add_exact_and_keyword_matches(&term_lower, context, &mut seed_entities);
        add_alias_matches(&term_lower, context, &mut seed_entities);
        add_substring_matches(&term_lower, context, &mut seed_entities);
        add_token_overlap_matches(&term_lower, context, &mut seed_entities);
    }
    seed_entities
}

fn hop_decay(hop: usize) -> f64 {
    match hop {
        0 => 1.0,
        1 => 0.5,
        _ => 0.25,
    }
}

fn score_tool_entity(
    entity_name: &str,
    score: f64,
    decay: f64,
    context: &ToolRelevanceContext<'_>,
    tool_scores: &mut HashMap<String, f64>,
) {
    if let Some(entity_id) = context.entities_by_name.get(entity_name)
        && let Some(entity) = context.entities.get(entity_id)
        && entity.entity_type == EntityType::Tool
    {
        let entry = tool_scores.entry(entity.name.clone()).or_insert(0.0);
        *entry = (*entry + score * decay).min(2.0);
    }
}

fn extend_outgoing_neighbors(
    entity_name: &str,
    score: f64,
    decay: f64,
    context: &ToolRelevanceContext<'_>,
    visited: &HashSet<String>,
    next_frontier: &mut Vec<(String, f64)>,
) {
    if let Some(rel_ids) = context.outgoing.get(entity_name) {
        for rel_id in rel_ids {
            if let Some(rel) = context.relations.get(rel_id)
                && !visited.contains(&rel.target)
            {
                let bonus = if rel.relation_type == RelationType::Contains {
                    0.2
                } else {
                    0.0
                };
                next_frontier.push((rel.target.clone(), score * decay + bonus));
            }
        }
    }
}

fn extend_incoming_neighbors(
    entity_name: &str,
    score: f64,
    decay: f64,
    context: &ToolRelevanceContext<'_>,
    visited: &HashSet<String>,
    next_frontier: &mut Vec<(String, f64)>,
) {
    if let Some(rel_ids) = context.incoming.get(entity_name) {
        for rel_id in rel_ids {
            if let Some(rel) = context.relations.get(rel_id)
                && !visited.contains(&rel.source)
            {
                next_frontier.push((rel.source.clone(), score * decay));
            }
        }
    }
}

fn collect_seed_tool_scores(
    seed_name: &str,
    base_score: f64,
    max_hops: usize,
    context: &ToolRelevanceContext<'_>,
    tool_scores: &mut HashMap<String, f64>,
) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut frontier: Vec<(String, f64)> = vec![(seed_name.to_string(), base_score)];

    for hop in 0..max_hops {
        let decay = hop_decay(hop);
        let mut next_frontier: Vec<(String, f64)> = Vec::new();

        for (entity_name, score) in &frontier {
            if visited.contains(entity_name) {
                continue;
            }
            visited.insert(entity_name.clone());

            score_tool_entity(entity_name, *score, decay, context, tool_scores);
            extend_outgoing_neighbors(
                entity_name,
                *score,
                decay,
                context,
                &visited,
                &mut next_frontier,
            );
            extend_incoming_neighbors(
                entity_name,
                *score,
                decay,
                context,
                &visited,
                &mut next_frontier,
            );
        }

        if next_frontier.is_empty() {
            break;
        }
        frontier = next_frontier;
    }
}

fn collect_tool_scores(
    seed_entities: &[(String, f64)],
    max_hops: usize,
    context: &ToolRelevanceContext<'_>,
) -> HashMap<String, f64> {
    let mut tool_scores: HashMap<String, f64> = HashMap::new();
    for (seed_name, base_score) in seed_entities {
        collect_seed_tool_scores(seed_name, *base_score, max_hops, context, &mut tool_scores);
    }
    tool_scores
}

fn sort_and_truncate_tool_scores(
    tool_scores: HashMap<String, f64>,
    limit: usize,
) -> Vec<(String, f64)> {
    let mut results: Vec<(String, f64)> = tool_scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    results
}

impl KnowledgeGraph {
    /// Query-time tool relevance scoring.
    ///
    /// Given a set of query terms, find `TOOL` entities connected to those terms
    /// via the `KnowledgeGraph` and return a relevance score for each tool.
    ///
    /// Algorithm:
    /// 1. For each query term, search for matching entities (exact + keyword + substring + fuzzy).
    /// 2. From each matched entity, walk outgoing/incoming relations (1-2 hops).
    /// 3. Collect all reachable `TOOL` entities and accumulate a score based on
    ///    hop distance and relation type.
    ///
    /// Returns `Vec<(tool_name, score)>` sorted by score descending, capped at `limit`.
    #[must_use]
    pub fn query_tool_relevance(
        &self,
        query_terms: &[String],
        max_hops: usize,
        limit: usize,
    ) -> Vec<(String, f64)> {
        let entities_by_name = read_lock(&self.entities_by_name);
        let entities = read_lock(&self.entities);
        let outgoing = read_lock(&self.outgoing_relations);
        let incoming = read_lock(&self.incoming_relations);
        let relations = read_lock(&self.relations);
        let context = ToolRelevanceContext {
            entities_by_name: &entities_by_name,
            entities: &entities,
            outgoing: &outgoing,
            incoming: &incoming,
            relations: &relations,
        };
        let seed_entities = collect_seed_entities(query_terms, &context);

        if seed_entities.is_empty() {
            return Vec::new();
        }

        let tool_scores = collect_tool_scores(&seed_entities, max_hops, &context);
        sort_and_truncate_tool_scores(tool_scores, limit)
    }
}
